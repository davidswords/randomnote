//! Encrypted task storage, unlocked by the account password.

mod email;
mod error;

use std::collections::HashMap;

use randomtask_crypto::{
    EncryptedItem, EncryptedItemsKey, ItemsKey, KeyParams, decrypt_item, decrypt_items_key,
    derive_root_key, encrypt_item, encrypt_items_key,
};
use randomtask_store::{Account, EncryptedStore, StoredItem, StoredItemsKey, Task};
use serde::{Deserialize, Serialize};

pub use error::VaultError;
use error::{corrupt, storage};

const MINIMUM_PASSWORD_CHARACTERS: usize = 12;
const PROTOCOL_VERSION: &str = "004";

/// The account's tasks, readable only once the vault is unlocked with the
/// password. The keys live only in memory and are wiped when it locks.
pub struct Vault {
    store: EncryptedStore,
    keys: Option<Keys>,
}

struct Keys {
    default_id: String,
    by_id: HashMap<String, ItemsKey>,
}

/// What a task's encrypted content holds.
#[derive(Serialize, Deserialize)]
struct TaskContent {
    title: String,
}

impl Vault {
    /// Wraps a store. The vault starts locked.
    pub fn new(store: EncryptedStore) -> Self {
        Self { store, keys: None }
    }

    pub fn has_account(&self) -> Result<bool, VaultError> {
        Ok(self.store.account().map_err(storage)?.is_some())
    }

    /// Creates the device's account, then unlocks the vault.
    /// The email is stored trimmed and in lower case.
    pub fn create_account(&mut self, email: &str, password: &str) -> Result<(), VaultError> {
        let email = email::normalise(email).ok_or(VaultError::InvalidEmail)?;
        if password.chars().count() < MINIMUM_PASSWORD_CHARACTERS {
            return Err(VaultError::PasswordTooShort);
        }
        if self.has_account()? {
            return Err(VaultError::AccountExists);
        }
        let account = Account {
            identifier: email,
            seed: random_hex::<32>()?,
            version: PROTOCOL_VERSION.to_owned(),
        };
        let root_key = derive_root_key(password, &key_params(&account)).map_err(corrupt)?;
        let items_key = ItemsKey::generate(random_hex::<16>()?).map_err(corrupt)?;
        let encrypted = encrypt_items_key(&items_key, &root_key).map_err(corrupt)?;
        self.store.create_account(&account).map_err(storage)?;
        self.store
            .save_items_key(&StoredItemsKey {
                id: encrypted.id,
                enc_item_key: encrypted.enc_item_key,
                content: encrypted.content,
            })
            .map_err(storage)?;
        self.keys = Some(Keys::from(vec![items_key]));
        self.encrypt_unencrypted_tasks()
    }

    /// Unlocks the vault. A wrong password leaves it locked.
    pub fn unlock(&mut self, password: &str) -> Result<(), VaultError> {
        let account = self.store.account().map_err(storage)?;
        let account = account.ok_or(VaultError::NoAccount)?;
        let root_key = derive_root_key(password, &key_params(&account)).map_err(corrupt)?;
        let items_keys = self
            .store
            .items_keys()
            .map_err(storage)?
            .into_iter()
            .map(|stored| {
                let encrypted = EncryptedItemsKey {
                    id: stored.id,
                    enc_item_key: stored.enc_item_key,
                    content: stored.content,
                };
                decrypt_items_key(&encrypted, &root_key).map_err(|_| VaultError::WrongPassword)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.keys = Some(Keys::from(items_keys));
        self.encrypt_unencrypted_tasks()
    }

    /// Forgets the keys, so tasks can't be read until the next unlock.
    pub fn lock(&mut self) {
        self.keys = None;
    }

    /// Saves a task, replacing any task with the same ID. Without an account
    /// the task is stored unencrypted, as nothing can encrypt it yet.
    pub fn save_task(&self, task: &Task) -> Result<(), VaultError> {
        let Access::Encrypted(keys) = self.access()? else {
            return self.store.save_unencrypted_task(task).map_err(storage);
        };
        let content = serde_json::to_string(&TaskContent {
            title: task.title.clone(),
        })
        .map_err(corrupt)?;
        let encrypted = encrypt_item(&task.id, &content, keys.default()).map_err(corrupt)?;
        self.store
            .save_item(&StoredItem {
                id: task.id.clone(),
                items_key_id: encrypted.items_key_id,
                enc_item_key: encrypted.enc_item_key,
                content: encrypted.content,
            })
            .map_err(storage)
    }

    pub fn find_task(&self, id: &str) -> Result<Option<Task>, VaultError> {
        let Access::Encrypted(keys) = self.access()? else {
            return self.store.find_unencrypted_task(id).map_err(storage);
        };
        let item = self.store.find_item(id).map_err(storage)?;
        item.map(|item| keys.decrypt_task(item)).transpose()
    }

    pub fn list_tasks(&self) -> Result<Vec<Task>, VaultError> {
        let Access::Encrypted(keys) = self.access()? else {
            return self.store.unencrypted_tasks().map_err(storage);
        };
        let items = self.store.items().map_err(storage)?;
        items
            .into_iter()
            .map(|item| keys.decrypt_task(item))
            .collect()
    }

    /// How tasks are reached: encrypted once unlocked, unencrypted while there
    /// is no account, and not at all while an account is locked.
    fn access(&self) -> Result<Access<'_>, VaultError> {
        match &self.keys {
            Some(keys) => Ok(Access::Encrypted(keys)),
            None if self.has_account()? => Err(VaultError::Locked),
            None => Ok(Access::Unencrypted),
        }
    }

    /// Encrypts any tasks saved before the account existed, then wipes the
    /// unencrypted copies. Run on unlocking too, in case an earlier run was
    /// interrupted.
    fn encrypt_unencrypted_tasks(&self) -> Result<(), VaultError> {
        let tasks = self.store.unencrypted_tasks().map_err(storage)?;
        if tasks.is_empty() {
            return Ok(());
        }
        tasks.iter().try_for_each(|task| self.save_task(task))?;
        self.store.wipe_unencrypted_tasks().map_err(storage)
    }
}

enum Access<'a> {
    Unencrypted,
    Encrypted(&'a Keys),
}

impl Keys {
    fn from(items_keys: Vec<ItemsKey>) -> Self {
        Self {
            default_id: items_keys[0].id().to_owned(),
            by_id: items_keys
                .into_iter()
                .map(|key| (key.id().to_owned(), key))
                .collect(),
        }
    }

    fn default(&self) -> &ItemsKey {
        &self.by_id[&self.default_id]
    }

    fn decrypt_task(&self, item: StoredItem) -> Result<Task, VaultError> {
        let items_key = self
            .by_id
            .get(&item.items_key_id)
            .ok_or_else(|| VaultError::Corrupt("unknown items key".to_owned()))?;
        let encrypted = EncryptedItem {
            items_key_id: item.items_key_id,
            enc_item_key: item.enc_item_key,
            content: item.content,
        };
        let content = decrypt_item(&item.id, &encrypted, items_key).map_err(corrupt)?;
        let content: TaskContent = serde_json::from_str(&content).map_err(corrupt)?;
        Ok(Task {
            id: item.id,
            title: content.title,
        })
    }
}

fn key_params(account: &Account) -> KeyParams {
    KeyParams {
        identifier: account.identifier.clone(),
        seed: account.seed.clone(),
    }
}

fn random_hex<const BYTES: usize>() -> Result<String, VaultError> {
    let mut bytes = [0u8; BYTES];
    getrandom::fill(&mut bytes).map_err(corrupt)?;
    Ok(hex::encode(bytes))
}

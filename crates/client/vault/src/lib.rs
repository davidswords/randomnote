//! Encrypted task storage, unlocked by the account password.

mod error;
mod server;

use std::collections::HashMap;

use randomtask_crypto::{
    EncryptedItem, EncryptedItemsKey, ItemsKey, RootKey, decrypt_item, decrypt_items_key,
    derive_root_key, encrypt_item, encrypt_items_key,
};
use randomtask_store::{Account, EncryptedStore, StoredItem, StoredItemsKey, Task};
use serde::{Deserialize, Serialize};

pub use error::VaultError;
use error::{corrupt, storage};
use server::key_params;
pub use server::{NewAccount, Registration, Session};
use zeroize::Zeroizing;

const MINIMUM_PASSWORD_CHARACTERS: usize = 12;
const PROTOCOL_VERSION: &str = "004";
/// The ID the server session is encrypted under.
const SESSION_ID: &str = "session";

/// The account's tasks, readable only once the vault is unlocked with the
/// password. The keys live only in memory and are wiped when it locks.
pub struct Vault {
    store: EncryptedStore,
    keys: Option<Keys>,
}

struct Keys {
    default_id: String,
    by_id: HashMap<String, ItemsKey>,
    /// Kept while unlocked, to register a device-only account with the server.
    server_password: Zeroizing<[u8; 32]>,
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

    /// Creates an account on this device only, then unlocks the vault.
    /// The email is stored trimmed and in lower case.
    pub fn create_account(&mut self, email: &str, password: &str) -> Result<(), VaultError> {
        let account = self.prepare_account(email, password)?;
        self.store_account(account)
    }

    /// Derives a new account's keys without storing anything, so the app can
    /// register it with the server before `create_registered_account`.
    pub fn prepare_account(&self, email: &str, password: &str) -> Result<NewAccount, VaultError> {
        let email = randomtask_crypto::normalise_email(email).ok_or(VaultError::InvalidEmail)?;
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
        Ok(NewAccount { account, root_key })
    }

    /// Stores an account the server has registered, with its session, then
    /// unlocks the vault.
    pub fn create_registered_account(
        &mut self,
        account: NewAccount,
        session: &Session,
    ) -> Result<(), VaultError> {
        self.store_account(account)?;
        self.save_session(session)
    }

    fn store_account(&mut self, new: NewAccount) -> Result<(), VaultError> {
        if self.has_account()? {
            return Err(VaultError::AccountExists);
        }
        let NewAccount { account, root_key } = new;
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
        self.keys = Some(Keys::new(vec![items_key], &root_key));
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
        self.keys = Some(Keys::new(items_keys, &root_key));
        self.encrypt_unencrypted_tasks()
    }

    /// What the app needs to register this account with the server, if it
    /// has no session yet: an account made on the device before there was a
    /// server. None without an account or once registered.
    pub fn pending_registration(&self) -> Result<Option<Registration>, VaultError> {
        let Some(account) = self.store.account().map_err(storage)? else {
            return Ok(None);
        };
        let keys = self.keys.as_ref().ok_or(VaultError::Locked)?;
        if self.store.session().map_err(storage)?.is_some() {
            return Ok(None);
        }
        Ok(Some(Registration {
            key_params: key_params(&account),
            server_password: keys.server_password.clone(),
        }))
    }

    /// Keeps the server session, encrypted, replacing any earlier one.
    pub fn save_session(&self, session: &Session) -> Result<(), VaultError> {
        let keys = self.unlocked_keys()?;
        let content = serde_json::to_string(session).expect("a session serialises");
        let encrypted = encrypt_item(SESSION_ID, &content, keys.default()).map_err(corrupt)?;
        self.store
            .save_session(&StoredItem {
                id: SESSION_ID.to_owned(),
                items_key_id: encrypted.items_key_id,
                enc_item_key: encrypted.enc_item_key,
                content: encrypted.content,
            })
            .map_err(storage)
    }

    /// The server session, if the account has one. None without an account.
    pub fn session(&self) -> Result<Option<Session>, VaultError> {
        let Access::Encrypted(keys) = self.access()? else {
            return Ok(None);
        };
        let stored = self.store.session().map_err(storage)?;
        stored
            .map(|item| {
                let content = keys.decrypt(item)?;
                serde_json::from_str(&content).map_err(corrupt)
            })
            .transpose()
    }

    fn unlocked_keys(&self) -> Result<&Keys, VaultError> {
        match self.access()? {
            Access::Encrypted(keys) => Ok(keys),
            Access::Unencrypted => Err(VaultError::NoAccount),
        }
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
    fn new(items_keys: Vec<ItemsKey>, root_key: &RootKey) -> Self {
        Self {
            server_password: Zeroizing::new(*root_key.server_password()),
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
        let id = item.id.clone();
        let content = self.decrypt(item)?;
        let content: TaskContent = serde_json::from_str(&content).map_err(corrupt)?;
        Ok(Task {
            id,
            title: content.title,
        })
    }

    fn decrypt(&self, item: StoredItem) -> Result<String, VaultError> {
        let items_key = self
            .by_id
            .get(&item.items_key_id)
            .ok_or_else(|| VaultError::Corrupt("unknown items key".to_owned()))?;
        let encrypted = EncryptedItem {
            items_key_id: item.items_key_id,
            enc_item_key: item.enc_item_key,
            content: item.content,
        };
        decrypt_item(&item.id, &encrypted, items_key).map_err(corrupt)
    }
}

fn random_hex<const BYTES: usize>() -> Result<String, VaultError> {
    let mut bytes = [0u8; BYTES];
    getrandom::fill(&mut bytes).map_err(corrupt)?;
    Ok(hex::encode(bytes))
}

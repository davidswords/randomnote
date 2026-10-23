//! What the vault gives the app for putting the account on the server, and
//! what it keeps from the server in return. The vault itself never talks to
//! the server.

use randomtask_crypto::{KeyParams, RootKey};
use randomtask_store::Account;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// A signed-in session on the server. Stored encrypted, like the tasks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    /// Milliseconds since the Unix epoch.
    pub access_expiration: i64,
    pub refresh_expiration: i64,
}

/// A new account's keys, derived but not yet stored, so the app can register
/// it with the server first.
pub struct NewAccount {
    pub(crate) account: Account,
    pub(crate) root_key: RootKey,
}

impl NewAccount {
    pub fn key_params(&self) -> KeyParams {
        key_params(&self.account)
    }

    /// Proves the password to the server. The master key never leaves the vault.
    pub fn server_password(&self) -> &[u8; 32] {
        self.root_key.server_password()
    }
}

/// What the app needs to register an existing account with the server.
pub struct Registration {
    pub(crate) key_params: KeyParams,
    pub(crate) server_password: Zeroizing<[u8; 32]>,
}

impl Registration {
    pub fn key_params(&self) -> KeyParams {
        self.key_params.clone()
    }

    pub fn server_password(&self) -> &[u8; 32] {
        &self.server_password
    }
}

pub(crate) fn key_params(account: &Account) -> KeyParams {
    KeyParams {
        identifier: account.identifier.clone(),
        seed: account.seed.clone(),
    }
}

//! SQLite storage for randomtask.

mod encrypted;

pub use encrypted::{Account, EncryptedStore, StoredItem, StoredItemsKey};

use serde::{Deserialize, Serialize};

/// A task as the app sees it, once decrypted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
}

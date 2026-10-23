//! Encryption for randomtask, following the Standard Notes 004 protocol.

mod email;
mod error;
mod hierarchy;
mod keys;
mod record;

pub use email::normalise_email;
pub use error::CryptoError;
pub use hierarchy::{
    EncryptedItem, EncryptedItemsKey, ItemsKey, decrypt_item, decrypt_items_key, encrypt_item,
    encrypt_items_key,
};
pub use keys::{KeyParams, RootKey, derive_root_key, salt};
pub use record::{decrypt, encrypt, encrypt_with_nonce};

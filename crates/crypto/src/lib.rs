//! Encryption for randomnote, following the Standard Notes 004 protocol.

mod error;
mod keys;
mod record;

pub use error::CryptoError;
pub use keys::{KeyParams, RootKey, derive_root_key, salt};
pub use record::{decrypt, encrypt, encrypt_with_nonce};

//! Deriving the root key from the password.

use argon2::{Algorithm, Argon2, Params, Version};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{CryptoError, to_error};

/// Argon2id memory cost, in KiB (64 MiB).
const MEMORY_KIB: u32 = 64 * 1024;
const ITERATIONS: u32 = 5;
const PARALLELISM: u32 = 1;
const KEY_BYTES: usize = 32;

/// The public values a root key is derived from, alongside the password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyParams {
    /// The account's identifier: its email address.
    pub identifier: String,
    /// A random 256-bit seed, as 64 hex characters, made when the account is created.
    pub seed: String,
}

/// The two halves derived from the password. Wiped from memory when dropped.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct RootKey {
    master_key: [u8; KEY_BYTES],
    server_password: [u8; KEY_BYTES],
}

impl RootKey {
    /// Encrypts the items keys. Never leaves the device.
    pub fn master_key(&self) -> &[u8; KEY_BYTES] {
        &self.master_key
    }

    /// Proves the password to a server. Unused until there is one.
    pub fn server_password(&self) -> &[u8; KEY_BYTES] {
        &self.server_password
    }
}

/// The salt: the first 128 bits, as hex, of SHA-256 of `identifier:seed`.
pub fn salt(params: &KeyParams) -> String {
    let digest = Sha256::digest(format!("{}:{}", params.identifier, params.seed));
    hex::encode(digest)[..32].to_owned()
}

/// Derives the root key from the password with Argon2id.
pub fn derive_root_key(password: &str, params: &KeyParams) -> Result<RootKey, CryptoError> {
    let salt = hex::decode(salt(params)).map_err(to_error)?;
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(MEMORY_KIB, ITERATIONS, PARALLELISM, Some(2 * KEY_BYTES)).map_err(to_error)?,
    );
    let mut derived = [0u8; 2 * KEY_BYTES];
    argon2
        .hash_password_into(password.as_bytes(), &salt, &mut derived)
        .map_err(to_error)?;
    let root_key = split(&derived);
    derived.zeroize();
    Ok(root_key)
}

fn split(derived: &[u8; 2 * KEY_BYTES]) -> RootKey {
    let mut root_key = RootKey {
        master_key: [0; KEY_BYTES],
        server_password: [0; KEY_BYTES],
    };
    root_key.master_key.copy_from_slice(&derived[..KEY_BYTES]);
    root_key
        .server_password
        .copy_from_slice(&derived[KEY_BYTES..]);
    root_key
}

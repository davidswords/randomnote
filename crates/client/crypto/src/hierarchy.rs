//! The key hierarchy: the master key encrypts items keys, items keys encrypt
//! a fresh key per item, and that item key encrypts the item's content.

use serde_json::{Value, json};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::error::{CryptoError, to_error};
use crate::keys::RootKey;
use crate::record::{decrypt, encrypt};

const KEY_BYTES: usize = 32;

/// A random key that encrypts the per-item keys. Wiped from memory when dropped.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ItemsKey {
    id: String,
    key: [u8; KEY_BYTES],
}

impl ItemsKey {
    /// Makes a new random items key with the given ID.
    pub fn generate(id: impl Into<String>) -> Result<Self, CryptoError> {
        Ok(Self {
            id: id.into(),
            key: random_key()?,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

/// An item as stored: which items key it uses, its encrypted key and content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedItem {
    pub items_key_id: String,
    pub enc_item_key: String,
    pub content: String,
}

/// An items key as stored, encrypted under the master key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedItemsKey {
    pub id: String,
    pub enc_item_key: String,
    pub content: String,
}

/// Encrypts an item's content under its own new key, wrapped by `items_key`.
pub fn encrypt_item(
    item_id: &str,
    plaintext: &str,
    items_key: &ItemsKey,
) -> Result<EncryptedItem, CryptoError> {
    let (enc_item_key, content) = seal(item_id, plaintext, &items_key.key)?;
    Ok(EncryptedItem {
        items_key_id: items_key.id.clone(),
        enc_item_key,
        content,
    })
}

/// Decrypts an item with the items key it was encrypted under. Any other
/// items key fails authentication.
pub fn decrypt_item(
    item_id: &str,
    encrypted: &EncryptedItem,
    items_key: &ItemsKey,
) -> Result<String, CryptoError> {
    open(
        item_id,
        &encrypted.enc_item_key,
        &encrypted.content,
        &items_key.key,
    )
}

/// Encrypts an items key under the master key, for storage.
pub fn encrypt_items_key(
    items_key: &ItemsKey,
    root_key: &RootKey,
) -> Result<EncryptedItemsKey, CryptoError> {
    let content = Zeroizing::new(
        json!({ "itemsKey": hex::encode(items_key.key), "version": "004" }).to_string(),
    );
    let (enc_item_key, content) = seal(&items_key.id, &content, root_key.master_key())?;
    Ok(EncryptedItemsKey {
        id: items_key.id.clone(),
        enc_item_key,
        content,
    })
}

/// Decrypts a stored items key with the master key.
pub fn decrypt_items_key(
    encrypted: &EncryptedItemsKey,
    root_key: &RootKey,
) -> Result<ItemsKey, CryptoError> {
    let content = Zeroizing::new(open(
        &encrypted.id,
        &encrypted.enc_item_key,
        &encrypted.content,
        root_key.master_key(),
    )?);
    let parsed: Value = serde_json::from_str(&content).map_err(to_error)?;
    let key_hex = parsed["itemsKey"]
        .as_str()
        .ok_or_else(|| CryptoError::new("items key content has no key"))?;
    Ok(ItemsKey {
        id: encrypted.id.clone(),
        key: key_from_hex(key_hex)?,
    })
}

/// Encrypts `plaintext` with a fresh item key, and that key with `wrapping_key`.
fn seal(
    item_id: &str,
    plaintext: &str,
    wrapping_key: &[u8; KEY_BYTES],
) -> Result<(String, String), CryptoError> {
    let item_key = Zeroizing::new(random_key()?);
    let content = encrypt(plaintext, &item_key, item_id)?;
    let item_key_hex = Zeroizing::new(hex::encode(*item_key));
    let enc_item_key = encrypt(&item_key_hex, wrapping_key, item_id)?;
    Ok((enc_item_key, content))
}

/// Reverses [`seal`].
fn open(
    item_id: &str,
    enc_item_key: &str,
    content: &str,
    wrapping_key: &[u8; KEY_BYTES],
) -> Result<String, CryptoError> {
    let item_key_hex = Zeroizing::new(decrypt(enc_item_key, wrapping_key, item_id)?);
    let item_key = Zeroizing::new(key_from_hex(&item_key_hex)?);
    decrypt(content, &item_key, item_id)
}

fn random_key() -> Result<[u8; KEY_BYTES], CryptoError> {
    let mut key = [0u8; KEY_BYTES];
    getrandom::fill(&mut key).map_err(to_error)?;
    Ok(key)
}

fn key_from_hex(key_hex: &str) -> Result<[u8; KEY_BYTES], CryptoError> {
    hex::decode(key_hex)
        .map_err(to_error)?
        .try_into()
        .map_err(|_| CryptoError::new("invalid key length"))
}

//! Encrypting one record as `004:nonce:ciphertext:authenticated-data`.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce};
use serde_json::{Value, json};

use crate::error::{CryptoError, to_error};

const VERSION: &str = "004";
const NONCE_BYTES: usize = 24;

/// Encrypts `plaintext` for the record `item_id` with a fresh random nonce.
pub fn encrypt(plaintext: &str, key: &[u8; 32], item_id: &str) -> Result<String, CryptoError> {
    let mut nonce = [0u8; NONCE_BYTES];
    getrandom::fill(&mut nonce).map_err(to_error)?;
    encrypt_with_nonce(plaintext, key, item_id, &nonce)
}

/// Encrypts with the given nonce. A nonce must never be reused with the same
/// key; outside of tests, use [`encrypt`].
pub fn encrypt_with_nonce(
    plaintext: &str,
    key: &[u8; 32],
    item_id: &str,
    nonce: &[u8; NONCE_BYTES],
) -> Result<String, CryptoError> {
    let authenticated_data = encode_authenticated_data(item_id);
    let ciphertext = cipher(key)
        .encrypt(
            &XNonce::from(*nonce),
            Payload {
                msg: plaintext.as_bytes(),
                aad: authenticated_data.as_bytes(),
            },
        )
        .map_err(to_error)?;
    Ok([
        VERSION,
        &hex::encode(nonce),
        &BASE64.encode(ciphertext),
        &authenticated_data,
    ]
    .join(":"))
}

/// Decrypts a record, checking it was encrypted for `item_id` and not tampered with.
pub fn decrypt(encrypted: &str, key: &[u8; 32], item_id: &str) -> Result<String, CryptoError> {
    let [version, nonce, ciphertext, authenticated_data] = split(encrypted)?;
    if version != VERSION {
        return Err(CryptoError::new("unsupported protocol version"));
    }
    check_belongs_to(authenticated_data, item_id)?;
    let nonce: [u8; NONCE_BYTES] = hex::decode(nonce)
        .map_err(to_error)?
        .try_into()
        .map_err(|_| CryptoError::new("invalid nonce length"))?;
    let plaintext = cipher(key)
        .decrypt(
            &XNonce::from(nonce),
            Payload {
                msg: &BASE64.decode(ciphertext).map_err(to_error)?,
                aad: authenticated_data.as_bytes(),
            },
        )
        .map_err(|_| CryptoError::new("decryption failed: wrong key or tampered record"))?;
    String::from_utf8(plaintext).map_err(to_error)
}

fn cipher(key: &[u8; 32]) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new(&Key::from(*key))
}

fn encode_authenticated_data(item_id: &str) -> String {
    BASE64.encode(json!({ "u": item_id, "v": VERSION }).to_string())
}

fn split(encrypted: &str) -> Result<[&str; 4], CryptoError> {
    encrypted
        .split(':')
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| CryptoError::new("not an encrypted record"))
}

fn check_belongs_to(authenticated_data: &str, item_id: &str) -> Result<(), CryptoError> {
    let decoded: Value =
        serde_json::from_slice(&BASE64.decode(authenticated_data).map_err(to_error)?)
            .map_err(to_error)?;
    if decoded["u"] != item_id || decoded["v"] != VERSION {
        return Err(CryptoError::new("record belongs to another item"));
    }
    Ok(())
}

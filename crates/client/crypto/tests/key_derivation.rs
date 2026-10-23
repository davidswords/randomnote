//! Key derivation as in Standard Notes protocol 004. Expected values come
//! from the reference Argon2 C library (through Python's argon2-cffi), not
//! from this crate.

use randomtask_crypto::{KeyParams, derive_root_key, salt};

fn params(identifier: &str, seed: &str) -> KeyParams {
    KeyParams {
        identifier: identifier.to_owned(),
        seed: seed.to_owned(),
    }
}

#[test]
fn salt_is_the_first_128_bits_of_sha256_of_identifier_and_seed() {
    assert_eq!(
        salt(&params("david", &"f".repeat(64))),
        "c87a63bd3bfeb1bebdd87305b9c0b9fa"
    );
}

#[test]
fn derives_the_master_key_and_server_password_from_the_password() {
    let root_key = derive_root_key(
        "correct horse battery staple",
        &params("david", &"f".repeat(64)),
    )
    .unwrap();

    assert_eq!(
        hex::encode(root_key.master_key()),
        "ca8f2588c8c2180224086f5fbd6755b20ff591e09ee141af126d356d11729a1a"
    );
    assert_eq!(
        hex::encode(root_key.server_password()),
        "aace833a2e1f1182024bb79cbecada55f7820d0b66f172b682cbd5b1617f1824"
    );
}

#[test]
fn handles_any_unicode_password_and_identifier() {
    let root_key = derive_root_key(
        "pässwörd 🔑",
        &params("someone@example.com", &"0123456789abcdef".repeat(4)),
    )
    .unwrap();

    assert_eq!(
        hex::encode(root_key.master_key()),
        "88afb9f9370a660b72060f0bf0fc1e087ca0f754eaa77fc4389d1815867c7072"
    );
    assert_eq!(
        hex::encode(root_key.server_password()),
        "f791d9c1618d15ef60a1deadee048919eac65512d942d7a824d9ebc2c73cd845"
    );
}

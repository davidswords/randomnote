//! Encrypting a record in the Standard Notes 004 string format. Expected
//! values come from libsodium (through Python's PyNaCl), the library Standard
//! Notes itself uses, not from this crate.

use randomnote_crypto::{decrypt, encrypt, encrypt_with_nonce};

const KEY: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
];
const NONCE: [u8; 24] = [
    100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118,
    119, 120, 121, 122, 123,
];
const MILK: &str = r#"{"title":"Buy milk"}"#;
const MILK_ENCRYPTED: &str = "004:6465666768696a6b6c6d6e6f707172737475767778797a7b:B1GN2YtC7KC9Et9K30/QoDfQcAxYfr5o3plOkLVeTY2Yxlql:eyJ1IjoidGFzay0xIiwidiI6IjAwNCJ9";

fn with_part(encrypted: &str, index: usize, replacement: &str) -> String {
    let mut parts: Vec<&str> = encrypted.split(':').collect();
    parts[index] = replacement;
    parts.join(":")
}

#[test]
fn encrypts_in_the_004_format_exactly_as_libsodium_does() {
    assert_eq!(
        encrypt_with_nonce(MILK, &KEY, "task-1", &NONCE).unwrap(),
        MILK_ENCRYPTED
    );
}

#[test]
fn encrypts_any_unicode_text_exactly_as_libsodium_does() {
    assert_eq!(
        encrypt_with_nonce(
            r#"{"title":"Café ☕ 🛒"}"#,
            &KEY,
            "019a0bca-5200-7000-8000-000000000000",
            &NONCE
        )
        .unwrap(),
        "004:6465666768696a6b6c6d6e6f707172737475767778797a7b:B1GN2YtC7KC9Et5ewKwU6bkjx1EUs1EirijMYflek/uGqFS7nhM2VxJQ:eyJ1IjoiMDE5YTBiY2EtNTIwMC03MDAwLTgwMDAtMDAwMDAwMDAwMDAwIiwidiI6IjAwNCJ9"
    );
}

#[test]
fn decrypts_a_record_encrypted_by_libsodium() {
    assert_eq!(decrypt(MILK_ENCRYPTED, &KEY, "task-1").unwrap(), MILK);
}

#[test]
fn decrypts_what_it_encrypts() {
    let encrypted = encrypt(MILK, &KEY, "task-1").unwrap();

    assert_eq!(decrypt(&encrypted, &KEY, "task-1").unwrap(), MILK);
}

#[test]
fn uses_a_fresh_nonce_every_time() {
    let first = encrypt(MILK, &KEY, "task-1").unwrap();
    let second = encrypt(MILK, &KEY, "task-1").unwrap();

    assert_ne!(first, second);
}

#[test]
fn rejects_a_tampered_ciphertext() {
    let tampered = with_part(
        MILK_ENCRYPTED,
        2,
        "C1GN2YtC7KC9Et9K30/QoDfQcAxYfr5o3plOkLVeTY2Yxlql",
    );

    assert!(decrypt(&tampered, &KEY, "task-1").is_err());
}

#[test]
fn rejects_the_wrong_key() {
    let wrong_key = [7u8; 32];

    assert!(decrypt(MILK_ENCRYPTED, &wrong_key, "task-1").is_err());
}

#[test]
fn rejects_a_record_moved_under_another_id() {
    assert!(decrypt(MILK_ENCRYPTED, &KEY, "task-2").is_err());
}

#[test]
fn rejects_a_record_whose_authenticated_data_was_rewritten_to_another_id() {
    let encrypted = encrypt(MILK, &KEY, "task-1").unwrap();
    // base64 of {"u":"task-2","v":"004"}
    let rewritten = with_part(&encrypted, 3, "eyJ1IjoidGFzay0yIiwidiI6IjAwNCJ9");

    assert!(decrypt(&rewritten, &KEY, "task-2").is_err());
}

#[test]
fn rejects_another_protocol_version() {
    let other_version = with_part(MILK_ENCRYPTED, 0, "003");

    assert!(decrypt(&other_version, &KEY, "task-1").is_err());
}

#[test]
fn rejects_text_that_is_not_an_encrypted_record() {
    assert!(decrypt("not encrypted", &KEY, "task-1").is_err());
}

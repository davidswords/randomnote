//! The Standard Notes 004 key hierarchy: the master key encrypts items keys,
//! items keys encrypt per-item keys, and per-item keys encrypt content.

use randomnote_crypto::{
    EncryptedItemsKey, ItemsKey, KeyParams, RootKey, decrypt_item, decrypt_items_key,
    derive_root_key, encrypt, encrypt_item, encrypt_items_key,
};

fn root_key(password: &str) -> RootKey {
    let params = KeyParams {
        identifier: "david".to_owned(),
        seed: "f".repeat(64),
    };
    derive_root_key(password, &params).unwrap()
}

const MILK: &str = r#"{"title":"Buy milk"}"#;

#[test]
fn decrypts_an_item_encrypted_under_an_items_key() {
    let items_key = ItemsKey::generate("items-key-1").unwrap();

    let encrypted = encrypt_item("task-1", MILK, &items_key).unwrap();

    assert_eq!(
        decrypt_item("task-1", &encrypted, &items_key).unwrap(),
        MILK
    );
}

#[test]
fn records_which_items_key_encrypted_the_item() {
    let items_key = ItemsKey::generate("items-key-1").unwrap();

    let encrypted = encrypt_item("task-1", MILK, &items_key).unwrap();

    assert_eq!(encrypted.items_key_id, "items-key-1");
}

#[test]
fn rejects_a_different_items_key() {
    let encrypted =
        encrypt_item("task-1", MILK, &ItemsKey::generate("items-key-1").unwrap()).unwrap();
    let impostor = ItemsKey::generate("items-key-1").unwrap();

    assert!(decrypt_item("task-1", &encrypted, &impostor).is_err());
}

#[test]
fn rejects_an_items_key_other_than_the_one_recorded() {
    let items_key = ItemsKey::generate("items-key-1").unwrap();
    let encrypted = encrypt_item("task-1", MILK, &items_key).unwrap();
    let other = ItemsKey::generate("items-key-2").unwrap();

    assert!(decrypt_item("task-1", &encrypted, &other).is_err());
}

#[test]
fn rejects_an_item_key_taken_from_another_item() {
    let items_key = ItemsKey::generate("items-key-1").unwrap();
    let mut milk = encrypt_item("task-1", MILK, &items_key).unwrap();
    let dentist = encrypt_item("task-2", r#"{"title":"Call the dentist"}"#, &items_key).unwrap();

    milk.enc_item_key = dentist.enc_item_key;

    assert!(decrypt_item("task-1", &milk, &items_key).is_err());
}

#[test]
fn gives_every_item_its_own_key() {
    let items_key = ItemsKey::generate("items-key-1").unwrap();
    let mut milk = encrypt_item("task-1", MILK, &items_key).unwrap();
    let other_milk = encrypt_item("task-1", MILK, &items_key).unwrap();

    milk.enc_item_key = other_milk.enc_item_key;

    assert!(decrypt_item("task-1", &milk, &items_key).is_err());
}

#[test]
fn makes_a_different_random_key_for_each_items_key() {
    let first = ItemsKey::generate("items-key-1").unwrap();
    let second = ItemsKey::generate("items-key-1").unwrap();
    let encrypted = encrypt_item("task-1", MILK, &first).unwrap();

    assert!(decrypt_item("task-1", &encrypted, &second).is_err());
}

#[test]
fn stores_an_items_key_encrypted_under_the_master_key() {
    let root_key = root_key("correct horse battery staple");
    let items_key = ItemsKey::generate("items-key-1").unwrap();
    let item = encrypt_item("task-1", MILK, &items_key).unwrap();

    let stored = encrypt_items_key(&items_key, &root_key).unwrap();
    let restored = decrypt_items_key(&stored, &root_key).unwrap();

    assert_eq!(restored.id(), "items-key-1");
    assert_eq!(decrypt_item("task-1", &item, &restored).unwrap(), MILK);
}

#[test]
fn cannot_unlock_an_items_key_with_the_wrong_password() {
    let items_key = ItemsKey::generate("items-key-1").unwrap();
    let stored = encrypt_items_key(&items_key, &root_key("correct horse battery staple")).unwrap();

    assert!(decrypt_items_key(&stored, &root_key("wrong password")).is_err());
}

/// An items key encrypted correctly, but holding the wrong content, as a
/// buggy version could write it.
fn malformed_items_key(root_key: &RootKey, item_key_hex: &str, content: &str) -> EncryptedItemsKey {
    let item_key: [u8; 32] = [9; 32];
    EncryptedItemsKey {
        id: "items-key-1".to_owned(),
        enc_item_key: encrypt(item_key_hex, root_key.master_key(), "items-key-1").unwrap(),
        content: encrypt(content, &item_key, "items-key-1").unwrap(),
    }
}

#[test]
fn rejects_an_items_key_whose_content_has_no_key() {
    let root_key = root_key("correct horse battery staple");
    let stored = malformed_items_key(&root_key, &"09".repeat(32), r#"{"version":"004"}"#);

    let error = decrypt_items_key(&stored, &root_key).err().unwrap();

    assert_eq!(error.to_string(), "items key content has no key");
}

#[test]
fn rejects_an_item_key_of_the_wrong_length() {
    let root_key = root_key("correct horse battery staple");
    let stored = malformed_items_key(&root_key, "abcd", r#"{"version":"004"}"#);

    let error = decrypt_items_key(&stored, &root_key).err().unwrap();

    assert_eq!(error.to_string(), "invalid key length");
}

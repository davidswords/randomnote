//! The vault against a real database and real encryption.

use std::path::Path;

use randomtask_store::{EncryptedStore, StoredItem, Task};
use randomtask_vault::{Vault, VaultError};

const PASSWORD: &str = "correct horse battery staple";

fn a_task(id: &str, title: &str) -> Task {
    Task {
        id: id.to_owned(),
        title: title.to_owned(),
    }
}

fn a_vault() -> Vault {
    Vault::new(EncryptedStore::open_in_memory().unwrap())
}

fn an_unlocked_vault() -> Vault {
    let mut vault = a_vault();
    vault.create_account("david", PASSWORD).unwrap();
    vault
}

fn open_vault(path: &Path) -> Vault {
    Vault::new(EncryptedStore::open(path).unwrap())
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[test]
fn has_no_account_until_one_is_created() {
    let vault = a_vault();

    assert_eq!(vault.has_account(), Ok(false));
}

#[test]
fn has_an_account_once_one_is_created() {
    let vault = an_unlocked_vault();

    assert_eq!(vault.has_account(), Ok(true));
}

#[test]
fn is_unlocked_after_creating_an_account() {
    let vault = an_unlocked_vault();

    vault.save_task(&a_task("task-1", "Buy milk")).unwrap();

    assert_eq!(vault.list_tasks(), Ok(vec![a_task("task-1", "Buy milk")]));
}

#[test]
fn accepts_a_password_of_exactly_12_characters() {
    let mut vault = a_vault();

    assert_eq!(vault.create_account("david", "123456789012"), Ok(()));
}

#[test]
fn refuses_a_password_shorter_than_12_characters_and_creates_no_account() {
    let mut vault = a_vault();

    assert_eq!(
        vault.create_account("david", "12345678901"),
        Err(VaultError::PasswordTooShort)
    );
    assert_eq!(vault.has_account(), Ok(false));
}

#[test]
fn counts_characters_rather_than_bytes_in_the_password() {
    let mut vault = a_vault();

    assert_eq!(
        vault.create_account("david", "ééééééééééé"),
        Err(VaultError::PasswordTooShort)
    );
}

#[test]
fn refuses_a_second_account() {
    let mut vault = an_unlocked_vault();

    assert_eq!(
        vault.create_account("someone-else", PASSWORD),
        Err(VaultError::AccountExists)
    );
}

#[test]
fn refuses_to_unlock_without_an_account() {
    let mut vault = a_vault();

    assert_eq!(vault.unlock(PASSWORD), Err(VaultError::NoAccount));
}

#[test]
fn refuses_task_access_while_locked() {
    let mut vault = an_unlocked_vault();

    vault.lock();

    assert_eq!(
        vault.save_task(&a_task("task-1", "Buy milk")),
        Err(VaultError::Locked)
    );
    assert_eq!(vault.find_task("task-1"), Err(VaultError::Locked));
    assert_eq!(vault.list_tasks(), Err(VaultError::Locked));
}

#[test]
fn unlocks_with_the_right_password_after_restarting() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    {
        let mut vault = open_vault(&path);
        vault.create_account("david", PASSWORD).unwrap();
        vault.save_task(&a_task("task-1", "Buy milk")).unwrap();
    }
    let mut reopened = open_vault(&path);

    reopened.unlock(PASSWORD).unwrap();

    assert_eq!(
        reopened.list_tasks(),
        Ok(vec![a_task("task-1", "Buy milk")])
    );
}

#[test]
fn refuses_the_wrong_password_and_stays_locked() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    {
        let mut vault = open_vault(&path);
        vault.create_account("david", PASSWORD).unwrap();
    }
    let mut reopened = open_vault(&path);

    assert_eq!(
        reopened.unlock("incorrect horse battery staple"),
        Err(VaultError::WrongPassword)
    );
    assert_eq!(reopened.list_tasks(), Err(VaultError::Locked));
}

#[test]
fn finds_a_saved_task_by_its_id_among_several() {
    let vault = an_unlocked_vault();
    vault.save_task(&a_task("task-1", "Buy milk")).unwrap();
    vault
        .save_task(&a_task("task-2", "Call the dentist"))
        .unwrap();

    assert_eq!(
        vault.find_task("task-2"),
        Ok(Some(a_task("task-2", "Call the dentist")))
    );
}

#[test]
fn finds_nothing_for_an_id_that_was_never_saved() {
    let vault = an_unlocked_vault();

    assert_eq!(vault.find_task("missing"), Ok(None));
}

#[test]
fn updates_a_task_in_place_when_it_is_saved_again() {
    let vault = an_unlocked_vault();
    vault.save_task(&a_task("task-1", "Buy milk")).unwrap();

    vault.save_task(&a_task("task-1", "Buy oat milk")).unwrap();

    assert_eq!(
        vault.list_tasks(),
        Ok(vec![a_task("task-1", "Buy oat milk")])
    );
}

#[test]
fn writes_no_task_title_or_password_to_the_database_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    {
        let mut vault = open_vault(&path);
        vault.create_account("david", PASSWORD).unwrap();
        vault.save_task(&a_task("task-1", "Buy milk")).unwrap();
    }

    let bytes = std::fs::read(&path).unwrap();

    assert!(!contains(&bytes, b"Buy milk"));
    assert!(!contains(&bytes, PASSWORD.as_bytes()));
}

#[test]
fn reports_a_storage_failure() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    vault.create_account("david", PASSWORD).unwrap();
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE items;")
        .unwrap();

    assert_eq!(
        vault.list_tasks(),
        Err(VaultError::Storage("no such table: items".to_owned()))
    );
}

#[test]
fn reports_a_tampered_task_as_corrupt() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    vault.create_account("david", PASSWORD).unwrap();
    vault.save_task(&a_task("task-1", "Buy milk")).unwrap();
    let store = EncryptedStore::open(&path).unwrap();
    let saved = store.find_item("task-1").unwrap().unwrap();
    store
        .save_item(&StoredItem {
            id: "task-2".to_owned(),
            ..saved
        })
        .unwrap();

    assert!(matches!(
        vault.find_task("task-2"),
        Err(VaultError::Corrupt(_))
    ));
}

#[test]
fn reports_a_task_whose_items_key_is_missing_as_corrupt() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    vault.create_account("david", PASSWORD).unwrap();
    vault.save_task(&a_task("task-1", "Buy milk")).unwrap();
    let store = EncryptedStore::open(&path).unwrap();
    let saved = store.find_item("task-1").unwrap().unwrap();
    store
        .save_item(&StoredItem {
            items_key_id: "missing".to_owned(),
            ..saved
        })
        .unwrap();

    assert_eq!(
        vault.list_tasks(),
        Err(VaultError::Corrupt("unknown items key".to_owned()))
    );
}

#[test]
fn explains_each_error_to_the_user() {
    assert_eq!(
        VaultError::PasswordTooShort.to_string(),
        "The password must be at least 12 characters."
    );
    assert_eq!(VaultError::WrongPassword.to_string(), "Wrong password");
    assert_eq!(
        VaultError::NoAccount.to_string(),
        "There is no account on this device."
    );
    assert_eq!(
        VaultError::AccountExists.to_string(),
        "An account already exists on this device."
    );
    assert_eq!(VaultError::Locked.to_string(), "The app is locked.");
    assert_eq!(
        VaultError::Storage("disk full".to_owned()).to_string(),
        "disk full"
    );
    assert_eq!(
        VaultError::Corrupt("unknown items key".to_owned()).to_string(),
        "unknown items key"
    );
}

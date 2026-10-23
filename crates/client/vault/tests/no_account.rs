//! Using the vault before there is an account, and securing it afterwards,
//! against a real database and real encryption.

use std::path::Path;

use randomtask_store::{EncryptedStore, Task};
use randomtask_vault::{Vault, VaultError};

const EMAIL: &str = "david@example.com";
const PASSWORD: &str = "correct horse battery staple";

fn a_task(id: &str, title: &str) -> Task {
    Task {
        id: id.to_owned(),
        title: title.to_owned(),
    }
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
fn saves_and_finds_tasks_without_an_account() {
    let vault = Vault::new(EncryptedStore::open_in_memory().unwrap());

    vault.save_task(&a_task("task-1", "Buy milk")).unwrap();
    vault
        .save_task(&a_task("task-2", "Call the dentist"))
        .unwrap();

    assert_eq!(
        vault.find_task("task-2"),
        Ok(Some(a_task("task-2", "Call the dentist")))
    );
    assert_eq!(vault.list_tasks().unwrap().len(), 2);
}

#[test]
fn keeps_tasks_without_an_account_across_restarts_with_no_password() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    open_vault(&path)
        .save_task(&a_task("task-1", "Buy milk"))
        .unwrap();

    let reopened = open_vault(&path);

    assert_eq!(
        reopened.list_tasks(),
        Ok(vec![a_task("task-1", "Buy milk")])
    );
}

#[test]
fn keeps_existing_tasks_when_an_account_is_created() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    vault.save_task(&a_task("task-1", "Buy milk")).unwrap();

    vault.create_account(EMAIL, PASSWORD).unwrap();

    assert_eq!(vault.list_tasks(), Ok(vec![a_task("task-1", "Buy milk")]));
    let mut reopened = open_vault(&path);
    assert_eq!(reopened.list_tasks(), Err(VaultError::Locked));
    reopened.unlock(PASSWORD).unwrap();
    assert_eq!(
        reopened.list_tasks(),
        Ok(vec![a_task("task-1", "Buy milk")])
    );
}

#[test]
fn leaves_no_unencrypted_copy_once_an_account_is_created() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    vault
        .save_task(&a_task("task-1", "Buy oat milk for the office"))
        .unwrap();

    vault.create_account(EMAIL, PASSWORD).unwrap();
    drop(vault);

    let store = EncryptedStore::open(&path).unwrap();
    assert_eq!(store.unencrypted_tasks().unwrap(), vec![]);
    drop(store);
    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, b"Buy oat milk for the office"));
}

#[test]
fn encrypts_tasks_left_unencrypted_when_it_is_next_unlocked() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    open_vault(&path).create_account(EMAIL, PASSWORD).unwrap();
    EncryptedStore::open(&path)
        .unwrap()
        .save_unencrypted_task(&a_task("task-1", "Buy milk"))
        .unwrap();
    let mut vault = open_vault(&path);

    vault.unlock(PASSWORD).unwrap();

    assert_eq!(vault.list_tasks(), Ok(vec![a_task("task-1", "Buy milk")]));
    drop(vault);
    let store = EncryptedStore::open(&path).unwrap();
    assert_eq!(store.unencrypted_tasks().unwrap(), vec![]);
}

#[test]
fn reports_a_storage_failure_without_an_account() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let vault = open_vault(&path);
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE account;")
        .unwrap();

    assert_eq!(
        vault.list_tasks(),
        Err(VaultError::Storage("no such table: account".to_owned()))
    );
}

#[test]
fn reports_a_failure_to_read_unencrypted_tasks() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE unencrypted_tasks;")
        .unwrap();

    assert_eq!(
        vault.save_task(&a_task("task-1", "Buy milk")),
        Err(VaultError::Storage(
            "no such table: unencrypted_tasks".to_owned()
        ))
    );
    assert!(vault.find_task("task-1").is_err());
    assert!(vault.list_tasks().is_err());
    assert_eq!(
        vault.create_account(EMAIL, PASSWORD),
        Err(VaultError::Storage(
            "no such table: unencrypted_tasks".to_owned()
        ))
    );
}

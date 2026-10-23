//! Putting the account on the server: preparing a new account's keys so the
//! app can register it, keeping the server session encrypted, and telling the
//! app when a device-only account still needs registering. Against a real
//! database and real encryption; the server itself is the app's concern.

use std::path::Path;

use randomtask_crypto::{KeyParams, derive_root_key};
use randomtask_store::{EncryptedStore, Task};
use randomtask_vault::{Session, Vault, VaultError};

const EMAIL: &str = "david@example.com";
const PASSWORD: &str = "correct horse battery staple";

fn a_session(access_token: &str) -> Session {
    Session {
        access_token: access_token.to_owned(),
        refresh_token: "1:session-uuid:refresh-secret-for-the-test".to_owned(),
        access_expiration: 1_000,
        refresh_expiration: 2_000,
    }
}

fn a_task(id: &str, title: &str) -> Task {
    Task {
        id: id.to_owned(),
        title: title.to_owned(),
    }
}

fn open_vault(path: &Path) -> Vault {
    Vault::new(EncryptedStore::open(path).unwrap())
}

fn a_vault() -> Vault {
    Vault::new(EncryptedStore::open_in_memory().unwrap())
}

fn server_password(password: &str, params: &KeyParams) -> [u8; 32] {
    *derive_root_key(password, params).unwrap().server_password()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[test]
fn prepares_a_new_account_s_key_params_and_server_password() {
    let vault = a_vault();

    let account = vault
        .prepare_account("  David@Example.COM ", PASSWORD)
        .unwrap();

    let params = account.key_params();
    assert_eq!(params.identifier, EMAIL);
    assert_eq!(params.seed.len(), 64);
    assert!(params.seed.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(
        *account.server_password(),
        server_password(PASSWORD, &params)
    );
}

#[test]
fn gives_each_prepared_account_its_own_seed() {
    let vault = a_vault();

    let first = vault.prepare_account(EMAIL, PASSWORD).unwrap();
    let second = vault.prepare_account(EMAIL, PASSWORD).unwrap();

    assert_ne!(first.key_params().seed, second.key_params().seed);
}

#[test]
fn stores_nothing_when_preparing_an_account() {
    let vault = a_vault();

    vault.prepare_account(EMAIL, PASSWORD).unwrap();

    assert_eq!(vault.has_account(), Ok(false));
}

#[test]
fn refuses_to_prepare_an_account_with_an_invalid_email_or_short_password() {
    let vault = a_vault();

    assert_eq!(
        vault.prepare_account("david", PASSWORD).err(),
        Some(VaultError::InvalidEmail)
    );
    assert_eq!(
        vault.prepare_account(EMAIL, "short").err(),
        Some(VaultError::PasswordTooShort)
    );
}

#[test]
fn refuses_to_prepare_an_account_when_one_exists() {
    let mut vault = a_vault();
    vault.create_account(EMAIL, PASSWORD).unwrap();

    assert_eq!(
        vault.prepare_account(EMAIL, PASSWORD).err(),
        Some(VaultError::AccountExists)
    );
}

#[test]
fn creates_a_registered_account_unlocked_with_its_session() {
    let mut vault = a_vault();
    vault.save_task(&a_task("task-1", "Buy milk")).unwrap();
    let account = vault.prepare_account(EMAIL, PASSWORD).unwrap();

    vault
        .create_registered_account(account, &a_session("1:a:access"))
        .unwrap();

    assert_eq!(vault.has_account(), Ok(true));
    assert_eq!(vault.session(), Ok(Some(a_session("1:a:access"))));
    assert_eq!(vault.list_tasks(), Ok(vec![a_task("task-1", "Buy milk")]));
    assert!(vault.pending_registration().unwrap().is_none());
}

#[test]
fn unlocks_a_registered_account_with_the_password_it_was_prepared_with() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    let account = vault.prepare_account(EMAIL, PASSWORD).unwrap();
    vault
        .create_registered_account(account, &a_session("1:a:access"))
        .unwrap();
    let mut reopened = open_vault(&path);

    reopened.unlock(PASSWORD).unwrap();

    assert_eq!(reopened.session(), Ok(Some(a_session("1:a:access"))));
    assert!(reopened.pending_registration().unwrap().is_none());
}

#[test]
fn refuses_to_create_a_registered_account_when_one_exists() {
    let mut vault = a_vault();
    let account = vault.prepare_account(EMAIL, PASSWORD).unwrap();
    vault.create_account(EMAIL, PASSWORD).unwrap();

    assert_eq!(
        vault.create_registered_account(account, &a_session("1:a:access")),
        Err(VaultError::AccountExists)
    );
}

#[test]
fn says_a_device_only_account_needs_registering_once_unlocked() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    open_vault(&path).create_account(EMAIL, PASSWORD).unwrap();
    let mut vault = open_vault(&path);
    vault.unlock(PASSWORD).unwrap();

    let registration = vault.pending_registration().unwrap().unwrap();

    let params = registration.key_params();
    assert_eq!(params.identifier, EMAIL);
    assert_eq!(
        *registration.server_password(),
        server_password(PASSWORD, &params)
    );
}

#[test]
fn stops_asking_for_registration_once_a_session_is_saved() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    vault.create_account(EMAIL, PASSWORD).unwrap();

    vault.save_session(&a_session("1:a:access")).unwrap();

    assert!(vault.pending_registration().unwrap().is_none());
    let mut reopened = open_vault(&path);
    reopened.unlock(PASSWORD).unwrap();
    assert!(reopened.pending_registration().unwrap().is_none());
    assert_eq!(reopened.session(), Ok(Some(a_session("1:a:access"))));
}

#[test]
fn replaces_the_session_when_a_new_one_is_saved() {
    let mut vault = a_vault();
    vault.create_account(EMAIL, PASSWORD).unwrap();
    vault.save_session(&a_session("1:a:first")).unwrap();

    vault.save_session(&a_session("1:a:second")).unwrap();

    assert_eq!(vault.session(), Ok(Some(a_session("1:a:second"))));
}

#[test]
fn needs_no_registration_and_has_no_session_without_an_account() {
    let vault = a_vault();

    assert!(vault.pending_registration().unwrap().is_none());
    assert_eq!(vault.session(), Ok(None));
    assert_eq!(
        vault.save_session(&a_session("1:a:access")),
        Err(VaultError::NoAccount)
    );
}

#[test]
fn keeps_the_session_and_registration_locked_away_while_locked() {
    let mut vault = a_vault();
    vault.create_account(EMAIL, PASSWORD).unwrap();

    vault.lock();

    assert!(matches!(
        vault.pending_registration(),
        Err(VaultError::Locked)
    ));
    assert_eq!(vault.session(), Err(VaultError::Locked));
    assert_eq!(
        vault.save_session(&a_session("1:a:access")),
        Err(VaultError::Locked)
    );
}

#[test]
fn stores_the_session_encrypted() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    vault.create_account(EMAIL, PASSWORD).unwrap();

    vault
        .save_session(&a_session("1:a:access-secret-for-the-test"))
        .unwrap();
    drop(vault);

    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, b"access-secret-for-the-test"));
    assert!(!contains(&bytes, b"refresh-secret-for-the-test"));
}

#[test]
fn reports_a_session_that_cannot_be_decrypted() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    vault.create_account(EMAIL, PASSWORD).unwrap();
    vault.save_session(&a_session("1:a:access")).unwrap();
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("UPDATE session SET content = '004:broken';")
        .unwrap();

    assert!(matches!(vault.session(), Err(VaultError::Corrupt(_))));
}

#[test]
fn reports_storage_failures_with_the_session() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let mut vault = open_vault(&path);
    vault.create_account(EMAIL, PASSWORD).unwrap();
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE session;")
        .unwrap();

    assert!(matches!(vault.session(), Err(VaultError::Storage(_))));
    assert!(matches!(
        vault.save_session(&a_session("1:a:access")),
        Err(VaultError::Storage(_))
    ));
    assert!(matches!(
        vault.pending_registration(),
        Err(VaultError::Storage(_))
    ));
}

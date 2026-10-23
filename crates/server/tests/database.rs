//! Opening the server's database file.

use randomtask_server::{Database, ServerError};

#[test]
fn reports_a_database_that_cannot_be_opened() {
    let directory = tempfile::tempdir().unwrap();

    let result = Database::open(&directory.path().join("missing").join("server.db"));

    assert!(matches!(result, Err(ServerError::Storage(_))));
}

#[test]
fn makes_a_256_bit_pseudo_key_the_first_time_it_is_asked() {
    let directory = tempfile::tempdir().unwrap();
    let database = Database::open(&directory.path().join("server.db")).unwrap();

    let key = database.pseudo_key().unwrap();

    assert_eq!(key.len(), 64, "{key}");
    assert!(key.chars().all(|c| c.is_ascii_hexdigit()), "{key}");
}

#[test]
fn keeps_the_same_pseudo_key_across_restarts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("server.db");
    let first = Database::open(&path).unwrap().pseudo_key().unwrap();

    let again = Database::open(&path).unwrap().pseudo_key().unwrap();

    assert_eq!(again, first);
}

#[test]
fn makes_a_different_pseudo_key_for_each_database() {
    let directory = tempfile::tempdir().unwrap();
    let first = Database::open(&directory.path().join("a.db")).unwrap();
    let second = Database::open(&directory.path().join("b.db")).unwrap();

    assert_ne!(first.pseudo_key().unwrap(), second.pseudo_key().unwrap());
}

#[test]
fn reports_a_storage_failure_when_getting_the_pseudo_key() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("server.db");
    let database = Database::open(&path).unwrap();
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE settings;")
        .unwrap();

    assert!(matches!(
        database.pseudo_key(),
        Err(ServerError::Storage(_))
    ));
}

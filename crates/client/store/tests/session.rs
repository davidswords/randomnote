//! The server session, stored as one encrypted record apart from the tasks,
//! against a real SQLite database.

use randomtask_store::{EncryptedStore, StoredItem};

fn a_session(content: &str) -> StoredItem {
    StoredItem {
        id: "session".to_owned(),
        items_key_id: "items-key-1".to_owned(),
        enc_item_key: "004:encrypted-key".to_owned(),
        content: content.to_owned(),
    }
}

#[test]
fn has_no_session_at_first() {
    let store = EncryptedStore::open_in_memory().unwrap();

    assert_eq!(store.session().unwrap(), None);
}

#[test]
fn keeps_the_session_it_was_given() {
    let store = EncryptedStore::open_in_memory().unwrap();

    store.save_session(&a_session("004:first")).unwrap();

    assert_eq!(store.session().unwrap(), Some(a_session("004:first")));
}

#[test]
fn replaces_the_session_when_given_a_new_one() {
    let store = EncryptedStore::open_in_memory().unwrap();
    store.save_session(&a_session("004:first")).unwrap();

    let second = StoredItem {
        id: "session-2".to_owned(),
        items_key_id: "items-key-2".to_owned(),
        enc_item_key: "004:another-key".to_owned(),
        content: "004:second".to_owned(),
    };

    store.save_session(&second).unwrap();

    assert_eq!(store.session().unwrap(), Some(second));
}

#[test]
fn keeps_the_session_apart_from_the_tasks() {
    let store = EncryptedStore::open_in_memory().unwrap();

    store.save_session(&a_session("004:first")).unwrap();

    assert_eq!(store.items().unwrap(), vec![]);
}

#[test]
fn keeps_the_session_after_the_file_is_closed_and_reopened() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    EncryptedStore::open(&path)
        .unwrap()
        .save_session(&a_session("004:first"))
        .unwrap();

    let reopened = EncryptedStore::open(&path).unwrap();

    assert_eq!(reopened.session().unwrap(), Some(a_session("004:first")));
}

#[test]
fn reports_errors_when_the_session_table_is_missing() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let store = EncryptedStore::open(&path).unwrap();
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE session;")
        .unwrap();

    assert!(store.save_session(&a_session("004:first")).is_err());
    assert!(store.session().is_err());
}

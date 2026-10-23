//! EncryptedStore against a real SQLite database.

use randomtask_store::{Account, EncryptedStore, StoredItem, StoredItemsKey, Task, TaskStore};

fn an_account() -> Account {
    Account {
        identifier: "david".to_owned(),
        seed: "f".repeat(64),
        version: "004".to_owned(),
    }
}

fn an_items_key(id: &str) -> StoredItemsKey {
    StoredItemsKey {
        id: id.to_owned(),
        enc_item_key: format!("004:{id}:key"),
        content: format!("004:{id}:content"),
    }
}

fn an_item(id: &str, content: &str) -> StoredItem {
    StoredItem {
        id: id.to_owned(),
        items_key_id: "items-key-1".to_owned(),
        enc_item_key: format!("004:{id}:key"),
        content: content.to_owned(),
    }
}

#[test]
fn has_no_account_until_one_is_created() {
    let store = EncryptedStore::open_in_memory().unwrap();

    assert_eq!(store.account().unwrap(), None);
}

#[test]
fn keeps_the_created_account() {
    let store = EncryptedStore::open_in_memory().unwrap();

    store.create_account(&an_account()).unwrap();

    assert_eq!(store.account().unwrap(), Some(an_account()));
}

#[test]
fn refuses_a_second_account() {
    let store = EncryptedStore::open_in_memory().unwrap();
    store.create_account(&an_account()).unwrap();

    let second = Account {
        identifier: "someone-else".to_owned(),
        ..an_account()
    };

    assert!(store.create_account(&second).is_err());
    assert_eq!(store.account().unwrap(), Some(an_account()));
}

#[test]
fn lists_no_items_keys_until_one_is_saved() {
    let store = EncryptedStore::open_in_memory().unwrap();

    assert_eq!(store.items_keys().unwrap(), vec![]);
}

#[test]
fn lists_every_saved_items_key() {
    let store = EncryptedStore::open_in_memory().unwrap();

    store.save_items_key(&an_items_key("items-key-1")).unwrap();
    store.save_items_key(&an_items_key("items-key-2")).unwrap();

    let keys = store.items_keys().unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&an_items_key("items-key-1")));
    assert!(keys.contains(&an_items_key("items-key-2")));
}

#[test]
fn finds_a_saved_item_by_its_id_among_several() {
    let store = EncryptedStore::open_in_memory().unwrap();
    store.save_item(&an_item("task-1", "004:milk")).unwrap();
    store.save_item(&an_item("task-2", "004:dentist")).unwrap();

    assert_eq!(
        store.find_item("task-2").unwrap(),
        Some(an_item("task-2", "004:dentist"))
    );
}

#[test]
fn finds_no_item_for_an_id_that_was_never_saved() {
    let store = EncryptedStore::open_in_memory().unwrap();

    assert_eq!(store.find_item("missing").unwrap(), None);
}

#[test]
fn lists_no_items_until_one_is_saved() {
    let store = EncryptedStore::open_in_memory().unwrap();

    assert_eq!(store.items().unwrap(), vec![]);
}

#[test]
fn lists_every_saved_item() {
    let store = EncryptedStore::open_in_memory().unwrap();

    store.save_item(&an_item("task-1", "004:milk")).unwrap();
    store.save_item(&an_item("task-2", "004:dentist")).unwrap();

    let items = store.items().unwrap();
    assert_eq!(items.len(), 2);
    assert!(items.contains(&an_item("task-1", "004:milk")));
    assert!(items.contains(&an_item("task-2", "004:dentist")));
}

#[test]
fn replaces_an_item_saved_again_with_the_same_id() {
    let store = EncryptedStore::open_in_memory().unwrap();
    store.save_item(&an_item("task-1", "004:milk")).unwrap();

    let renamed = StoredItem {
        items_key_id: "items-key-2".to_owned(),
        enc_item_key: "004:new-key".to_owned(),
        ..an_item("task-1", "004:oat-milk")
    };
    store.save_item(&renamed).unwrap();

    assert_eq!(store.items().unwrap(), vec![renamed]);
}

#[test]
fn keeps_everything_after_the_database_file_is_closed_and_reopened() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");

    {
        let store = EncryptedStore::open(&path).unwrap();
        store.create_account(&an_account()).unwrap();
        store.save_items_key(&an_items_key("items-key-1")).unwrap();
        store.save_item(&an_item("task-1", "004:milk")).unwrap();
    }
    let reopened = EncryptedStore::open(&path).unwrap();

    assert_eq!(reopened.account().unwrap(), Some(an_account()));
    assert_eq!(
        reopened.items_keys().unwrap(),
        vec![an_items_key("items-key-1")]
    );
    assert_eq!(
        reopened.items().unwrap(),
        vec![an_item("task-1", "004:milk")]
    );
}

#[test]
fn deletes_unencrypted_tasks_from_version_0_2_and_leaves_no_trace_in_the_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.db");
    {
        let legacy = TaskStore::open(&path).unwrap();
        legacy
            .save(&Task {
                id: "task-1".to_owned(),
                title: "Buy milk".to_owned(),
            })
            .unwrap();
    }

    EncryptedStore::open(&path).unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, b"Buy milk"));
    let tables: Vec<String> = rusqlite::Connection::open(&path)
        .unwrap()
        .prepare("SELECT name FROM sqlite_schema WHERE name = 'tasks'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(tables, Vec::<String>::new());
}

#[test]
fn reports_an_error_when_the_file_is_not_a_database() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("not-a-database.db");
    std::fs::write(
        &path,
        "this is not a database, just some text that fills the header",
    )
    .unwrap();

    assert!(EncryptedStore::open(&path).is_err());
}

#[test]
fn reports_errors_when_the_tables_are_missing() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let store = EncryptedStore::open(&path).unwrap();
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE account; DROP TABLE items_keys; DROP TABLE items;")
        .unwrap();

    assert!(store.account().is_err());
    assert!(store.create_account(&an_account()).is_err());
    assert!(store.items_keys().is_err());
    assert!(store.save_items_key(&an_items_key("items-key-1")).is_err());
    assert!(store.items().is_err());
    assert!(store.find_item("task-1").is_err());
    assert!(store.save_item(&an_item("task-1", "004:milk")).is_err());
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

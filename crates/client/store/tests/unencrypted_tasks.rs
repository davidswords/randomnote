//! Tasks stored before the user has an account, when nothing can encrypt
//! them yet, against a real SQLite database.

use randomtask_store::{EncryptedStore, Task};

fn a_task(id: &str, title: &str) -> Task {
    Task {
        id: id.to_owned(),
        title: title.to_owned(),
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[test]
fn lists_no_unencrypted_tasks_at_first() {
    let store = EncryptedStore::open_in_memory().unwrap();

    assert_eq!(store.unencrypted_tasks().unwrap(), vec![]);
}

#[test]
fn finds_an_unencrypted_task_by_its_id_among_several() {
    let store = EncryptedStore::open_in_memory().unwrap();
    store
        .save_unencrypted_task(&a_task("task-1", "Buy milk"))
        .unwrap();
    store
        .save_unencrypted_task(&a_task("task-2", "Call the dentist"))
        .unwrap();

    assert_eq!(
        store.find_unencrypted_task("task-2").unwrap(),
        Some(a_task("task-2", "Call the dentist"))
    );
    assert_eq!(store.find_unencrypted_task("missing").unwrap(), None);
}

#[test]
fn lists_every_unencrypted_task() {
    let store = EncryptedStore::open_in_memory().unwrap();
    store
        .save_unencrypted_task(&a_task("task-1", "Buy milk"))
        .unwrap();
    store
        .save_unencrypted_task(&a_task("task-2", "Call the dentist"))
        .unwrap();

    let tasks = store.unencrypted_tasks().unwrap();

    assert_eq!(tasks.len(), 2);
    assert!(tasks.contains(&a_task("task-1", "Buy milk")));
    assert!(tasks.contains(&a_task("task-2", "Call the dentist")));
}

#[test]
fn replaces_an_unencrypted_task_saved_again_with_the_same_id() {
    let store = EncryptedStore::open_in_memory().unwrap();
    store
        .save_unencrypted_task(&a_task("task-1", "Buy milk"))
        .unwrap();

    store
        .save_unencrypted_task(&a_task("task-1", "Buy oat milk"))
        .unwrap();

    assert_eq!(
        store.unencrypted_tasks().unwrap(),
        vec![a_task("task-1", "Buy oat milk")]
    );
}

#[test]
fn keeps_unencrypted_tasks_after_the_file_is_closed_and_reopened() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    {
        let store = EncryptedStore::open(&path).unwrap();
        store
            .save_unencrypted_task(&a_task("task-1", "Buy milk"))
            .unwrap();
    }

    let reopened = EncryptedStore::open(&path).unwrap();

    assert_eq!(
        reopened.unencrypted_tasks().unwrap(),
        vec![a_task("task-1", "Buy milk")]
    );
}

#[test]
fn wipes_unencrypted_tasks_so_no_trace_is_left_in_the_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let store = EncryptedStore::open(&path).unwrap();
    store
        .save_unencrypted_task(&a_task("task-1", "Buy oat milk for the office"))
        .unwrap();

    store.wipe_unencrypted_tasks().unwrap();

    assert_eq!(store.unencrypted_tasks().unwrap(), vec![]);
    drop(store);
    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, b"Buy oat milk for the office"));
}

#[test]
fn reports_errors_when_the_unencrypted_tasks_table_is_missing() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let store = EncryptedStore::open(&path).unwrap();
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE unencrypted_tasks;")
        .unwrap();

    assert!(
        store
            .save_unencrypted_task(&a_task("task-1", "Buy milk"))
            .is_err()
    );
    assert!(store.find_unencrypted_task("task-1").is_err());
    assert!(store.unencrypted_tasks().is_err());
    assert!(store.wipe_unencrypted_tasks().is_err());
}

//! The task commands called through Tauri's real IPC handling, against a real
//! vault. The messages match what TauriTaskRepository sends.

mod support;

use randomtask_store::EncryptedStore;
use serde_json::{Value, json};
use support::TestApp;

#[test]
fn find_task_returns_null_for_an_id_that_was_never_saved() {
    let app = TestApp::unlocked();

    assert_eq!(
        app.invoke("find_task", json!({ "id": "missing" })),
        Ok(Value::Null)
    );
}

#[test]
fn find_task_returns_a_task_saved_with_save_task() {
    let app = TestApp::unlocked();
    let task = json!({ "id": "task-1", "title": "Buy milk" });

    assert_eq!(
        app.invoke("save_task", json!({ "task": task })),
        Ok(Value::Null)
    );

    assert_eq!(app.invoke("find_task", json!({ "id": "task-1" })), Ok(task));
}

#[test]
fn list_tasks_returns_every_saved_task() {
    let app = TestApp::unlocked();
    let task = json!({ "id": "task-1", "title": "Buy milk" });
    app.invoke("save_task", json!({ "task": task })).unwrap();

    assert_eq!(app.invoke("list_tasks", json!({})), Ok(json!([task])));
}

#[test]
fn task_commands_are_refused_while_locked() {
    let app = TestApp::without_account();
    let task = json!({ "id": "task-1", "title": "Buy milk" });

    assert_eq!(
        app.invoke("save_task", json!({ "task": task })),
        Err(json!("The app is locked."))
    );
    assert_eq!(
        app.invoke("find_task", json!({ "id": "task-1" })),
        Err(json!("The app is locked."))
    );
    assert_eq!(
        app.invoke("list_tasks", json!({})),
        Err(json!("The app is locked."))
    );
}

#[test]
fn commands_return_storage_errors_as_messages() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let app = TestApp::with_store(EncryptedStore::open(&path).unwrap());
    app.create_account().unwrap();
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE items;")
        .unwrap();

    assert_eq!(
        app.invoke("list_tasks", json!({})),
        Err(json!("no such table: items"))
    );
}

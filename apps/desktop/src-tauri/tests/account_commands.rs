//! The account commands called through Tauri's real IPC handling, against a
//! real vault on a real disk.

mod support;

use randomtask_store::EncryptedStore;
use serde_json::{Value, json};
use support::{PASSWORD, TestApp};

#[test]
fn has_account_is_false_until_an_account_is_created() {
    let app = TestApp::without_account();

    assert_eq!(app.invoke("has_account", json!({})), Ok(json!(false)));
}

#[test]
fn create_account_creates_the_account_and_unlocks() {
    let app = TestApp::without_account();

    assert_eq!(app.create_account(), Ok(Value::Null));

    assert_eq!(app.invoke("has_account", json!({})), Ok(json!(true)));
    assert_eq!(app.invoke("list_tasks", json!({})), Ok(json!([])));
}

#[test]
fn create_account_explains_a_password_that_is_too_short() {
    let app = TestApp::without_account();

    assert_eq!(
        app.invoke(
            "create_account",
            json!({ "username": "david", "password": "too short" })
        ),
        Err(json!("The password must be at least 12 characters."))
    );
}

#[test]
fn unlock_with_the_right_password_gives_access_to_tasks_after_a_restart() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    let task = json!({ "id": "task-1", "title": "Buy milk" });
    {
        let app = TestApp::with_store(EncryptedStore::open(&path).unwrap());
        app.create_account().unwrap();
        app.invoke("save_task", json!({ "task": task })).unwrap();
    }
    let restarted = TestApp::with_store(EncryptedStore::open(&path).unwrap());

    assert_eq!(
        restarted.invoke("unlock", json!({ "password": PASSWORD })),
        Ok(Value::Null)
    );

    assert_eq!(restarted.invoke("list_tasks", json!({})), Ok(json!([task])));
}

#[test]
fn unlock_with_the_wrong_password_says_so() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    {
        let app = TestApp::with_store(EncryptedStore::open(&path).unwrap());
        app.create_account().unwrap();
    }
    let restarted = TestApp::with_store(EncryptedStore::open(&path).unwrap());

    assert_eq!(
        restarted.invoke("unlock", json!({ "password": "incorrect horse battery" })),
        Err(json!("Wrong password"))
    );
}

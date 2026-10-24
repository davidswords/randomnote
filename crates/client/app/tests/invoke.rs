//! The app's commands by name, with JSON arguments and results, as the web
//! app's worker passes them on. The names, argument keys and JSON match the
//! desktop app's Tauri commands, so both apps share the same adapters.

use randomtask_app::App;
use randomtask_store::EncryptedStore;
use randomtask_vault::Vault;
use serde_json::{Value, json};

const PASSWORD: &str = "correct horse battery staple";

fn app() -> App {
    App::new(Vault::new(EncryptedStore::open_in_memory().unwrap()), None)
}

#[tokio::test]
async fn has_account_is_false_until_an_account_is_created() {
    let app = app();

    assert_eq!(app.invoke("has_account", json!({})).await, Ok(json!(false)));
}

#[tokio::test]
async fn create_account_creates_the_account_and_unlocks() {
    let app = app();

    let result = app
        .invoke(
            "create_account",
            json!({ "email": "david@example.com", "password": PASSWORD }),
        )
        .await;

    assert_eq!(result, Ok(Value::Null));
    assert_eq!(app.invoke("has_account", json!({})).await, Ok(json!(true)));
}

#[tokio::test]
async fn create_account_passes_on_the_reason_it_was_refused() {
    let app = app();

    let result = app
        .invoke(
            "create_account",
            json!({ "email": "david@example.com", "password": "too short" }),
        )
        .await;

    assert_eq!(
        result,
        Err("The password must be at least 12 characters.".to_owned())
    );
}

#[tokio::test]
async fn unlock_with_the_wrong_password_says_so() {
    let app = app();
    app.create_account("david@example.com", PASSWORD)
        .await
        .unwrap();

    let result = app
        .invoke("unlock", json!({ "password": "the wrong password" }))
        .await;

    assert_eq!(result, Err("Wrong password".to_owned()));
}

#[tokio::test]
async fn unlock_with_the_right_password_succeeds() {
    let app = app();
    app.create_account("david@example.com", PASSWORD)
        .await
        .unwrap();

    let result = app.invoke("unlock", json!({ "password": PASSWORD })).await;

    assert_eq!(result, Ok(Value::Null));
}

#[tokio::test]
async fn saves_finds_and_lists_tasks() {
    let app = app();
    let task = json!({ "id": "task-1", "title": "Buy milk" });

    let saved = app.invoke("save_task", json!({ "task": task })).await;

    assert_eq!(saved, Ok(Value::Null));
    assert_eq!(
        app.invoke("find_task", json!({ "id": "task-1" })).await,
        Ok(task.clone())
    );
    assert_eq!(
        app.invoke("find_task", json!({ "id": "missing" })).await,
        Ok(Value::Null)
    );
    assert_eq!(app.invoke("list_tasks", json!({})).await, Ok(json!([task])));
}

#[tokio::test]
async fn task_commands_pass_on_the_reason_they_were_refused() {
    let app = app();
    app.create_account("david@example.com", PASSWORD)
        .await
        .unwrap();
    app.lock();
    let locked = Err("The app is locked.".to_owned());

    assert_eq!(
        app.invoke(
            "save_task",
            json!({ "task": { "id": "task-1", "title": "Buy milk" } })
        )
        .await,
        locked
    );
    assert_eq!(
        app.invoke("find_task", json!({ "id": "task-1" })).await,
        locked
    );
    assert_eq!(app.invoke("list_tasks", json!({})).await, locked);
    assert_eq!(app.invoke("has_account", json!({})).await, Ok(json!(true)));
}

#[tokio::test]
async fn refuses_an_unknown_command() {
    let app = app();

    assert_eq!(
        app.invoke("delete_everything", json!({})).await,
        Err("Unknown command: delete_everything".to_owned())
    );
}

#[tokio::test]
async fn refuses_missing_or_wrongly_shaped_arguments() {
    let app = app();

    for (command, args) in [
        ("create_account", json!({ "email": "david@example.com" })),
        ("unlock", json!({})),
        ("save_task", json!({ "task": { "id": "task-1" } })),
        ("find_task", json!({ "id": 7 })),
    ] {
        let result = app.invoke(command, args).await;

        let message = result.unwrap_err();
        assert!(
            message.starts_with(&format!("Invalid arguments for {command}: ")),
            "{message}"
        );
    }
}

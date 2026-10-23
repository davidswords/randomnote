//! The task commands called through Tauri's real IPC handling, against a real
//! in-memory SQLite store. The messages match what TauriTaskRepository sends.

use randomtask_lib::{TaskState, with_task_commands};
use randomtask_store::TaskStore;
use serde_json::{Value, json};
use tauri::test::{
    INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
};
use tauri::webview::InvokeRequest;
use tauri::{App, Manager, WebviewWindow, WebviewWindowBuilder};

struct TestApp {
    _app: App<MockRuntime>,
    webview: WebviewWindow<MockRuntime>,
}

impl TestApp {
    fn with_empty_store() -> Self {
        Self::with_store(TaskStore::open_in_memory().unwrap())
    }

    fn with_store(store: TaskStore) -> Self {
        let app = with_task_commands(mock_builder())
            .build(mock_context(noop_assets()))
            .unwrap();
        app.manage(TaskState::new(store));
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        Self { _app: app, webview }
    }

    fn invoke(&self, command: &str, args: Value) -> Result<Value, Value> {
        let request = InvokeRequest {
            cmd: command.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: args.into(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        };
        get_ipc_response(&self.webview, request).map(|body| body.deserialize().unwrap())
    }
}

#[test]
fn find_task_returns_null_for_an_id_that_was_never_saved() {
    let app = TestApp::with_empty_store();

    assert_eq!(
        app.invoke("find_task", json!({ "id": "missing" })),
        Ok(Value::Null)
    );
}

#[test]
fn find_task_returns_a_task_saved_with_save_task() {
    let app = TestApp::with_empty_store();
    let task = json!({ "id": "task-1", "title": "Buy milk" });

    assert_eq!(
        app.invoke("save_task", json!({ "task": task })),
        Ok(Value::Null)
    );

    assert_eq!(app.invoke("find_task", json!({ "id": "task-1" })), Ok(task));
}

#[test]
fn list_tasks_returns_every_saved_task() {
    let app = TestApp::with_empty_store();
    let task = json!({ "id": "task-1", "title": "Buy milk" });
    app.invoke("save_task", json!({ "task": task })).unwrap();

    assert_eq!(app.invoke("list_tasks", json!({})), Ok(json!([task])));
}

#[test]
fn commands_return_storage_errors_as_messages() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.db");
    let app = TestApp::with_store(TaskStore::open(&path).unwrap());
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE tasks;")
        .unwrap();

    assert_eq!(
        app.invoke("list_tasks", json!({})),
        Err(json!("no such table: tasks"))
    );
}

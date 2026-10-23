//! A mock Tauri app with the real commands and a real vault, for sending IPC
//! messages exactly as the web view does.

// Each test binary uses a different part of this module.
#![allow(dead_code)]

use randomtask_lib::{VaultState, with_commands};
use randomtask_store::EncryptedStore;
use randomtask_vault::Vault;
use serde_json::Value;
use tauri::test::{
    INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
};
use tauri::webview::InvokeRequest;
use tauri::{App, Manager, WebviewWindow, WebviewWindowBuilder};

pub const PASSWORD: &str = "correct horse battery staple";

pub struct TestApp {
    _app: App<MockRuntime>,
    webview: WebviewWindow<MockRuntime>,
}

impl TestApp {
    /// An app with a new, empty in-memory database and no account.
    pub fn without_account() -> Self {
        Self::with_store(EncryptedStore::open_in_memory().unwrap())
    }

    /// An app whose account has just been created, so it's unlocked.
    pub fn unlocked() -> Self {
        let app = Self::without_account();
        app.create_account().unwrap();
        app
    }

    pub fn with_store(store: EncryptedStore) -> Self {
        let app = with_commands(mock_builder())
            .build(mock_context(noop_assets()))
            .unwrap();
        app.manage(VaultState::new(Vault::new(store)));
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        Self { _app: app, webview }
    }

    pub fn create_account(&self) -> Result<Value, Value> {
        self.invoke(
            "create_account",
            serde_json::json!({ "email": "david@example.com", "password": PASSWORD }),
        )
    }

    pub fn invoke(&self, command: &str, args: Value) -> Result<Value, Value> {
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

//! The account commands with a server configured, through Tauri's real IPC,
//! against a real vault on disk and the real server on a free local port.
//! The app crate tests the rest of what signing up and unlocking do with a
//! server; these pin that the commands reach it and pass its messages on.

mod support;

use std::path::{Path, PathBuf};

use randomtask_server::{Accounts, Database, SystemClock, router};
use randomtask_store::EncryptedStore;
use serde_json::{Value, json};
use support::{PASSWORD, TestApp};
use tempfile::TempDir;

const EMAIL: &str = "david@example.com";

/// The real server, run on its own thread so the app's commands can call it.
struct TestServer {
    url: String,
    _directory: TempDir,
}

impl TestServer {
    fn start() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let database = Database::open(&directory.path().join("server.db")).unwrap();
        let accounts = Accounts::new(database, Box::new(SystemClock), "test-key".to_owned());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async move {
                    let listener = tokio::net::TcpListener::from_std(listener).unwrap();
                    axum::serve(listener, router(accounts)).await.unwrap();
                });
        });
        Self {
            url,
            _directory: directory,
        }
    }

    /// The seed the server gives for `email`: the real one once registered,
    /// a made-up one before.
    fn seed_for(&self, email: &str) -> String {
        let body: Value = post(
            &format!("{}/v2/login-params", self.url),
            json!({ "email": email, "code_challenge": "test" }),
        );
        body["pw_nonce"].as_str().unwrap().to_owned()
    }
}

fn post(url: &str, body: Value) -> Value {
    let url = url.to_owned();
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                reqwest::Client::new()
                    .post(url)
                    .json(&body)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap()
            })
    })
    .join()
    .unwrap()
}

/// A URL where nothing is listening.
fn unreachable_url() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    format!("http://{}", listener.local_addr().unwrap())
}

fn database() -> (TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    (directory, path)
}

fn store(path: &Path) -> EncryptedStore {
    EncryptedStore::open(path).unwrap()
}

fn local_seed(path: &Path) -> String {
    store(path).account().unwrap().unwrap().seed
}

fn has_session(path: &Path) -> bool {
    store(path).session().unwrap().is_some()
}

fn sign_up(app: &TestApp) -> Result<Value, Value> {
    app.invoke(
        "create_account",
        json!({ "email": EMAIL, "password": PASSWORD }),
    )
}

#[test]
fn signing_up_registers_the_account_with_the_server() {
    let server = TestServer::start();
    let (_directory, path) = database();
    let app = TestApp::with_server(store(&path), &server.url);

    assert_eq!(sign_up(&app), Ok(Value::Null));

    assert_eq!(app.invoke("has_account", json!({})), Ok(json!(true)));
    assert_eq!(server.seed_for(EMAIL), local_seed(&path));
    assert!(has_session(&path));
}

#[test]
fn signing_up_with_a_taken_email_says_so_and_creates_nothing() {
    let server = TestServer::start();
    let (_first, first_path) = database();
    sign_up(&TestApp::with_server(store(&first_path), &server.url)).unwrap();
    let (_directory, path) = database();
    let app = TestApp::with_server(store(&path), &server.url);

    let result = sign_up(&app);

    assert_eq!(result, Err(json!("This email is already registered.")));
    assert_eq!(app.invoke("has_account", json!({})), Ok(json!(false)));
}

#[test]
fn unlocking_registers_an_account_made_on_the_device() {
    let server = TestServer::start();
    let (_directory, path) = database();
    TestApp::with_store(store(&path)).create_account().unwrap();
    let app = TestApp::with_server(store(&path), &server.url);

    assert_eq!(
        app.invoke("unlock", json!({ "password": PASSWORD })),
        Ok(Value::Null)
    );

    assert_eq!(server.seed_for(EMAIL), local_seed(&path));
    assert!(has_session(&path));
}

#[test]
fn refuses_account_commands_with_missing_arguments() {
    let (_directory, path) = database();
    let app = TestApp::with_server(store(&path), &unreachable_url());

    for (command, args) in [
        ("create_account", json!({})),
        ("create_account", json!({ "email": EMAIL })),
        ("unlock", json!({})),
    ] {
        let result = app.invoke(command, args);

        let message = result.unwrap_err();
        assert!(
            message.as_str().unwrap().contains("missing required key"),
            "{command}: {message}"
        );
    }
}

//! The account commands with a server configured: signing up registers the
//! account with the real randomtask server, and unlocking registers accounts
//! made on the device before there was one. Through Tauri's real IPC, against
//! a real vault on disk and the real server on a free local port.

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
fn signing_up_keeps_the_tasks_made_before() {
    let server = TestServer::start();
    let (_directory, path) = database();
    let app = TestApp::with_server(store(&path), &server.url);
    let task = json!({ "id": "task-1", "title": "Buy milk" });
    app.invoke("save_task", json!({ "task": task })).unwrap();

    sign_up(&app).unwrap();

    assert_eq!(app.invoke("list_tasks", json!({})), Ok(json!([task])));
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
fn signing_up_without_the_server_says_so_and_leaves_tasks_unencrypted() {
    let (_directory, path) = database();
    let app = TestApp::with_server(store(&path), &unreachable_url());
    let task = json!({ "id": "task-1", "title": "Buy milk" });
    app.invoke("save_task", json!({ "task": task })).unwrap();

    let result = sign_up(&app);

    assert_eq!(
        result,
        Err(json!(
            "Can't reach the randomtask server. Check your connection and try again."
        ))
    );
    assert_eq!(app.invoke("has_account", json!({})), Ok(json!(false)));
    assert_eq!(app.invoke("list_tasks", json!({})), Ok(json!([task])));
}

#[test]
fn signing_up_checks_the_password_before_calling_the_server() {
    let (_directory, path) = database();
    let app = TestApp::with_server(store(&path), &unreachable_url());

    let result = app.invoke(
        "create_account",
        json!({ "email": EMAIL, "password": "too short" }),
    );

    assert_eq!(
        result,
        Err(json!("The password must be at least 12 characters."))
    );
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
fn unlocking_still_works_when_the_server_cannot_be_reached() {
    let (_directory, path) = database();
    TestApp::with_store(store(&path)).create_account().unwrap();
    let app = TestApp::with_server(store(&path), &unreachable_url());

    assert_eq!(
        app.invoke("unlock", json!({ "password": PASSWORD })),
        Ok(Value::Null)
    );

    assert_eq!(app.invoke("list_tasks", json!({})), Ok(json!([])));
    assert!(!has_session(&path));
}

#[test]
fn unlocking_registers_on_a_later_try_once_the_server_is_back() {
    let server = TestServer::start();
    let (_directory, path) = database();
    TestApp::with_store(store(&path)).create_account().unwrap();
    TestApp::with_server(store(&path), &unreachable_url())
        .invoke("unlock", json!({ "password": PASSWORD }))
        .unwrap();
    let app = TestApp::with_server(store(&path), &server.url);

    app.invoke("unlock", json!({ "password": PASSWORD }))
        .unwrap();

    assert_eq!(server.seed_for(EMAIL), local_seed(&path));
    assert!(has_session(&path));
}

#[test]
fn unlocking_signs_in_when_the_account_is_already_registered_with_the_same_keys() {
    let server = TestServer::start();
    let (_directory, path) = database();
    TestApp::with_server(store(&path), &server.url)
        .create_account()
        .unwrap();
    // As if the session were lost after registering, say by a crash.
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DELETE FROM session;")
        .unwrap();
    let app = TestApp::with_server(store(&path), &server.url);

    app.invoke("unlock", json!({ "password": PASSWORD }))
        .unwrap();

    assert!(has_session(&path));
}

#[test]
fn unlocking_leaves_an_account_unregistered_when_its_email_belongs_to_someone_else() {
    let server = TestServer::start();
    let (_other, other_path) = database();
    sign_up(&TestApp::with_server(store(&other_path), &server.url)).unwrap();
    let (_directory, path) = database();
    TestApp::with_store(store(&path)).create_account().unwrap();
    let app = TestApp::with_server(store(&path), &server.url);

    assert_eq!(
        app.invoke("unlock", json!({ "password": PASSWORD })),
        Ok(Value::Null)
    );

    assert!(!has_session(&path));
    assert_ne!(server.seed_for(EMAIL), local_seed(&path));
}

#[test]
fn unlocking_with_the_wrong_password_never_calls_the_server() {
    let server = TestServer::start();
    let (_directory, path) = database();
    TestApp::with_store(store(&path)).create_account().unwrap();
    let app = TestApp::with_server(store(&path), &server.url);

    assert_eq!(
        app.invoke("unlock", json!({ "password": "the wrong password" })),
        Err(json!("Wrong password"))
    );

    assert_ne!(server.seed_for(EMAIL), local_seed(&path));
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

#[test]
fn keeps_the_session_the_server_gave_when_signing_up() {
    let server = TestServer::start();
    let (_directory, path) = database();
    sign_up(&TestApp::with_server(store(&path), &server.url)).unwrap();
    let mut vault = randomtask_vault::Vault::new(store(&path));
    vault.unlock(PASSWORD).unwrap();

    let session = vault.session().unwrap().unwrap();

    const DAY_MS: i64 = 24 * 60 * 60 * 1000;
    assert_eq!(
        session.refresh_expiration - session.access_expiration,
        (365 - 60) * DAY_MS
    );
    assert!(session.access_token.starts_with("1:"));
    assert!(session.refresh_token.starts_with("1:"));
    assert_ne!(session.access_token, session.refresh_token);
}

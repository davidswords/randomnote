//! The app's account and tasks, with and without a server. Against a real
//! vault on disk and, where there is one, the real randomtask server on a
//! free local port.

use std::path::{Path, PathBuf};

use randomtask_api::ServerClient;
use randomtask_app::App;
use randomtask_server::{Accounts, Database, SystemClock, router};
use randomtask_store::{EncryptedStore, Task};
use randomtask_vault::Vault;
use serde_json::{Value, json};
use tempfile::TempDir;

const EMAIL: &str = "david@example.com";
const PASSWORD: &str = "correct horse battery staple";

struct TestServer {
    url: String,
    client: reqwest::Client,
    _directory: TempDir,
}

impl TestServer {
    async fn start() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let database = Database::open(&directory.path().join("server.db")).unwrap();
        let accounts = Accounts::new(database, Box::new(SystemClock), "test-key".to_owned());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(listener, router(accounts)).await.unwrap() });
        Self {
            url,
            client: reqwest::Client::new(),
            _directory: directory,
        }
    }

    /// The seed the server gives for `email`: the real one once registered,
    /// a made-up one before.
    async fn seed_for(&self, email: &str) -> String {
        let body: Value = self
            .client
            .post(format!("{}/v2/login-params", self.url))
            .json(&json!({ "email": email, "code_challenge": "test" }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        body["pw_nonce"].as_str().unwrap().to_owned()
    }
}

fn unreachable_url() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    format!("http://{}", listener.local_addr().unwrap())
}

fn database() -> (TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("randomtask.db");
    (directory, path)
}

fn app(path: &Path, server: Option<&str>) -> App {
    let vault = Vault::new(EncryptedStore::open(path).unwrap());
    App::new(vault, server.map(ServerClient::new))
}

fn local_seed(path: &Path) -> String {
    EncryptedStore::open(path)
        .unwrap()
        .account()
        .unwrap()
        .unwrap()
        .seed
}

fn has_session(path: &Path) -> bool {
    EncryptedStore::open(path)
        .unwrap()
        .session()
        .unwrap()
        .is_some()
}

fn a_task() -> Task {
    Task {
        id: "task-1".to_owned(),
        title: "Buy milk".to_owned(),
    }
}

#[tokio::test]
async fn keeps_tasks_without_an_account() {
    let (_directory, path) = database();
    let app = app(&path, None);

    app.save_task(&a_task()).unwrap();

    assert_eq!(app.has_account(), Ok(false));
    assert_eq!(app.find_task("task-1"), Ok(Some(a_task())));
    assert_eq!(app.list_tasks(), Ok(vec![a_task()]));
}

#[tokio::test]
async fn creates_an_account_on_the_device_without_a_server() {
    let (_directory, path) = database();
    let app = app(&path, None);

    app.create_account(EMAIL, PASSWORD).await.unwrap();

    assert_eq!(app.has_account(), Ok(true));
    assert!(!has_session(&path));
}

#[tokio::test]
async fn unlocks_an_account_on_the_device_without_a_server() {
    let (_directory, path) = database();
    app(&path, None)
        .create_account(EMAIL, PASSWORD)
        .await
        .unwrap();
    let app = app(&path, None);

    assert_eq!(
        app.list_tasks().map_err(|e| e.to_string()),
        Err("The app is locked.".to_owned())
    );
    app.unlock(PASSWORD).await.unwrap();

    assert_eq!(app.list_tasks(), Ok(vec![]));
}

#[tokio::test]
async fn signing_up_registers_the_account_with_the_server() {
    let server = TestServer::start().await;
    let (_directory, path) = database();
    let app = app(&path, Some(&server.url));
    app.save_task(&a_task()).unwrap();

    app.create_account(EMAIL, PASSWORD).await.unwrap();

    assert_eq!(server.seed_for(EMAIL).await, local_seed(&path));
    assert!(has_session(&path));
    assert_eq!(app.list_tasks(), Ok(vec![a_task()]));
}

#[tokio::test]
async fn keeps_the_session_the_server_gave() {
    let server = TestServer::start().await;
    let (_directory, path) = database();
    app(&path, Some(&server.url))
        .create_account(EMAIL, PASSWORD)
        .await
        .unwrap();
    let mut vault = Vault::new(EncryptedStore::open(&path).unwrap());
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

#[tokio::test]
async fn signing_up_with_a_taken_email_says_so_and_creates_nothing() {
    let server = TestServer::start().await;
    let (_first, first_path) = database();
    app(&first_path, Some(&server.url))
        .create_account(EMAIL, PASSWORD)
        .await
        .unwrap();
    let (_directory, path) = database();
    let app = app(&path, Some(&server.url));

    let result = app.create_account(EMAIL, PASSWORD).await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "This email is already registered."
    );
    assert_eq!(app.has_account(), Ok(false));
}

#[tokio::test]
async fn signing_up_without_the_server_says_so_and_leaves_tasks_unencrypted() {
    let (_directory, path) = database();
    let app = app(&path, Some(&unreachable_url()));
    app.save_task(&a_task()).unwrap();

    let result = app.create_account(EMAIL, PASSWORD).await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "Can't reach the randomtask server. Check your connection and try again."
    );
    assert_eq!(app.has_account(), Ok(false));
    assert_eq!(app.list_tasks(), Ok(vec![a_task()]));
}

#[tokio::test]
async fn signing_up_checks_the_password_before_calling_the_server() {
    let (_directory, path) = database();
    let app = app(&path, Some(&unreachable_url()));

    let result = app.create_account(EMAIL, "too short").await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "The password must be at least 12 characters."
    );
}

#[tokio::test]
async fn unlocking_registers_an_account_made_on_the_device() {
    let server = TestServer::start().await;
    let (_directory, path) = database();
    app(&path, None)
        .create_account(EMAIL, PASSWORD)
        .await
        .unwrap();
    let app = app(&path, Some(&server.url));

    app.unlock(PASSWORD).await.unwrap();

    assert_eq!(server.seed_for(EMAIL).await, local_seed(&path));
    assert!(has_session(&path));
}

#[tokio::test]
async fn unlocking_still_works_when_the_server_cannot_be_reached() {
    let (_directory, path) = database();
    app(&path, None)
        .create_account(EMAIL, PASSWORD)
        .await
        .unwrap();
    let app = app(&path, Some(&unreachable_url()));

    app.unlock(PASSWORD).await.unwrap();

    assert_eq!(app.list_tasks(), Ok(vec![]));
    assert!(!has_session(&path));
}

#[tokio::test]
async fn unlocking_registers_on_a_later_try_once_the_server_is_back() {
    let server = TestServer::start().await;
    let (_directory, path) = database();
    app(&path, None)
        .create_account(EMAIL, PASSWORD)
        .await
        .unwrap();
    app(&path, Some(&unreachable_url()))
        .unlock(PASSWORD)
        .await
        .unwrap();
    let app = app(&path, Some(&server.url));

    app.unlock(PASSWORD).await.unwrap();

    assert_eq!(server.seed_for(EMAIL).await, local_seed(&path));
    assert!(has_session(&path));
}

#[tokio::test]
async fn unlocking_signs_in_when_the_account_is_already_registered_with_the_same_keys() {
    let server = TestServer::start().await;
    let (_directory, path) = database();
    app(&path, Some(&server.url))
        .create_account(EMAIL, PASSWORD)
        .await
        .unwrap();
    // As if the session were lost after registering, say by a crash.
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DELETE FROM session;")
        .unwrap();
    let app = app(&path, Some(&server.url));

    app.unlock(PASSWORD).await.unwrap();

    assert!(has_session(&path));
}

#[tokio::test]
async fn unlocking_leaves_an_account_unregistered_when_its_email_belongs_to_someone_else() {
    let server = TestServer::start().await;
    let (_other, other_path) = database();
    app(&other_path, Some(&server.url))
        .create_account(EMAIL, PASSWORD)
        .await
        .unwrap();
    let (_directory, path) = database();
    app(&path, None)
        .create_account(EMAIL, PASSWORD)
        .await
        .unwrap();
    let app = app(&path, Some(&server.url));

    app.unlock(PASSWORD).await.unwrap();

    assert!(!has_session(&path));
    assert_ne!(server.seed_for(EMAIL).await, local_seed(&path));
}

#[tokio::test]
async fn unlocking_with_the_wrong_password_never_calls_the_server() {
    let server = TestServer::start().await;
    let (_directory, path) = database();
    app(&path, None)
        .create_account(EMAIL, PASSWORD)
        .await
        .unwrap();
    let app = app(&path, Some(&server.url));

    let result = app.unlock("the wrong password").await;

    assert_eq!(result.unwrap_err().to_string(), "Wrong password");
    assert_ne!(server.seed_for(EMAIL).await, local_seed(&path));
}

#[tokio::test]
async fn reports_storage_failures() {
    let (_directory, path) = database();
    let app = app(&path, None);
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE account;")
        .unwrap();

    assert!(app.has_account().is_err());
    assert!(app.save_task(&a_task()).is_err());
    assert!(app.find_task("task-1").is_err());
    assert!(app.list_tasks().is_err());
    assert!(app.create_account(EMAIL, PASSWORD).await.is_err());
    assert!(app.unlock(PASSWORD).await.is_err());
}

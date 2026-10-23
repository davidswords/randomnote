//! A real server on a free local port, backed by a real SQLite file.

use std::path::{Path, PathBuf};

use randomtask_server::{Accounts, Clock, Database, router};
use serde_json::{Value, json};
use tempfile::TempDir;

/// 2026-10-23T12:00:00Z, in milliseconds.
pub const NOW: i64 = 1_792_756_800_000;
pub const EMAIL: &str = "david@example.com";
/// The server password: the second half of the root key, as hex.
pub const SERVER_PASSWORD: &str =
    "a7b1c2d3e4f5061728394a5b6c7d8e9fa0b1c2d3e4f5061728394a5b6c7d8e9f";
pub const NONCE: &str = "1f2e3d4c5b6a79881f2e3d4c5b6a79881f2e3d4c5b6a79881f2e3d4c5b6a7988";

struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> i64 {
        NOW
    }
}

pub struct TestServer {
    pub url: String,
    pub client: reqwest::Client,
    directory: TempDir,
}

impl TestServer {
    pub async fn start() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let database = Database::open(&directory.path().join("server.db")).unwrap();
        let accounts = Accounts::new(database, Box::new(FixedClock));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(listener, router(accounts)).await.unwrap() });
        Self {
            url,
            client: reqwest::Client::new(),
            directory,
        }
    }

    pub fn database_path(&self) -> PathBuf {
        self.directory.path().join("server.db")
    }

    pub async fn post(&self, path: &str, body: Value) -> (u16, Value) {
        let response = self
            .client
            .post(format!("{}{path}", self.url))
            .json(&body)
            .send()
            .await
            .unwrap();
        (response.status().as_u16(), response.json().await.unwrap())
    }

    pub async fn register(&self, body: Value) -> (u16, Value) {
        self.post("/v1/users", body).await
    }
}

pub fn registration() -> Value {
    json!({
        "email": EMAIL,
        "password": SERVER_PASSWORD,
        "pw_nonce": NONCE,
        "version": "004",
        "origination": "registration",
        "created": "1792756800000",
        "api": "20200115",
        "ephemeral": false,
    })
}

pub fn with(field: &str, value: Value) -> Value {
    let mut body = registration();
    body[field] = value;
    body
}

pub fn file_contains(path: &Path, needle: &str) -> bool {
    let bytes = std::fs::read(path).unwrap();
    bytes
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
}

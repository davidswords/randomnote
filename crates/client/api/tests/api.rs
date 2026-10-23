//! The API client against the real randomtask server, plus small
//! fake servers for the responses the real one never gives.

use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::any;
use axum::routing::post;
use axum::{Json, Router};
use randomtask_api::{ClientError, ServerClient};
use randomtask_crypto::KeyParams;
use randomtask_server::{Accounts, Database, SystemClock, router};
use serde_json::{Value, json};
use tempfile::TempDir;

const EMAIL: &str = "david@example.com";
const SEED: &str = "1f2e3d4c5b6a79881f2e3d4c5b6a79881f2e3d4c5b6a79881f2e3d4c5b6a7988";
const SERVER_PASSWORD: [u8; 32] = [7; 32];

fn key_params(email: &str) -> KeyParams {
    KeyParams {
        identifier: email.to_owned(),
        seed: SEED.to_owned(),
    }
}

async fn serve(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}

/// The real server, with its database in a directory that lives as long as
/// the returned guard.
async fn real_server() -> (ServerClient, TempDir) {
    real_server_at(|url| url).await
}

async fn real_server_at(address: impl Fn(String) -> String) -> (ServerClient, TempDir) {
    let directory = tempfile::tempdir().unwrap();
    let database = Database::open(&directory.path().join("server.db")).unwrap();
    let accounts = Accounts::new(
        database,
        Box::new(SystemClock),
        "test-pseudo-key".to_owned(),
    );
    let url = serve(router(accounts)).await;
    (ServerClient::new(&address(url)), directory)
}

async fn fake_server(response: &'static str) -> ServerClient {
    fake_server_with_status(StatusCode::OK, response).await
}

async fn fake_server_with_status(status: StatusCode, response: &'static str) -> ServerClient {
    let respond = move || async move { (status, response) };
    let app = Router::new()
        .route("/v1/users", post(respond))
        .route("/v2/login-params", post(respond))
        .route("/v2/login", post(respond));
    ServerClient::new(&serve(app).await)
}

#[tokio::test]
async fn registers_an_account_and_starts_a_session() {
    let (client, _database) = real_server().await;

    let session = client
        .register(&key_params(EMAIL), &SERVER_PASSWORD)
        .await
        .unwrap();

    assert!(session.access_token.starts_with("1:"));
    assert!(session.refresh_token.starts_with("1:"));
    assert!(session.access_expiration < session.refresh_expiration);
}

#[tokio::test]
async fn gets_back_the_key_params_it_registered_with() {
    let (client, _database) = real_server().await;
    client
        .register(&key_params(EMAIL), &SERVER_PASSWORD)
        .await
        .unwrap();

    let challenge = client.key_params(EMAIL).await.unwrap();

    assert_eq!(challenge.key_params, key_params(EMAIL));
}

#[tokio::test]
async fn signs_in_with_the_server_password() {
    let (client, _database) = real_server().await;
    client
        .register(&key_params(EMAIL), &SERVER_PASSWORD)
        .await
        .unwrap();
    let challenge = client.key_params(EMAIL).await.unwrap();

    let session = client.sign_in(challenge, &SERVER_PASSWORD).await.unwrap();

    assert!(session.access_token.starts_with("1:"));
}

#[tokio::test]
async fn passes_on_the_server_s_message_when_it_refuses() {
    let (client, _database) = real_server().await;
    client
        .register(&key_params(EMAIL), &SERVER_PASSWORD)
        .await
        .unwrap();

    let taken = client.register(&key_params(EMAIL), &SERVER_PASSWORD).await;
    let challenge = client.key_params(EMAIL).await.unwrap();
    let wrong = client.sign_in(challenge, &[0; 32]).await;

    assert_eq!(
        taken.unwrap_err(),
        ClientError::Refused("This email is already registered.".to_owned())
    );
    assert_eq!(
        wrong.unwrap_err(),
        ClientError::Refused("Invalid email or password".to_owned())
    );
}

#[tokio::test]
async fn says_when_the_server_cannot_be_reached() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let client = ServerClient::new(&url);

    let registered = client.register(&key_params(EMAIL), &SERVER_PASSWORD).await;
    let params = client.key_params(EMAIL).await;

    assert_eq!(registered.unwrap_err(), ClientError::Unreachable);
    assert_eq!(params.unwrap_err(), ClientError::Unreachable);
    assert_eq!(
        ClientError::Unreachable.to_string(),
        "Can't reach the randomtask server. Check your connection and try again."
    );
}

#[tokio::test]
async fn refuses_key_params_for_a_protocol_version_other_than_004() {
    let client =
        fake_server(r#"{"identifier":"david@example.com","pw_nonce":"abc","version":"003"}"#).await;

    let params = client.key_params(EMAIL).await;

    assert_eq!(params.unwrap_err(), ClientError::UnsupportedVersion);
    assert_eq!(
        ClientError::UnsupportedVersion.to_string(),
        "This account needs a newer version of randomtask. Please update it."
    );
}

#[tokio::test]
async fn reports_a_response_it_does_not_understand() {
    let client = fake_server("not json").await;

    let registered = client.register(&key_params(EMAIL), &SERVER_PASSWORD).await;
    let params = client.key_params(EMAIL).await;

    assert!(matches!(
        registered,
        Err(ClientError::UnexpectedResponse(_))
    ));
    assert!(matches!(params, Err(ClientError::UnexpectedResponse(_))));
    assert_eq!(
        ClientError::UnexpectedResponse("detail".to_owned()).to_string(),
        "The randomtask server sent a response the app doesn't understand."
    );
}

#[tokio::test]
async fn shows_the_user_the_server_s_own_message() {
    let refused = ClientError::Refused("This email is already registered.".to_owned());

    assert_eq!(refused.to_string(), "This email is already registered.");
}

#[tokio::test]
async fn reports_an_error_response_without_a_message() {
    let client = fake_server_with_status(StatusCode::BAD_REQUEST, r#"{"error":{}}"#).await;

    let registered = client.register(&key_params(EMAIL), &SERVER_PASSWORD).await;

    assert_eq!(
        registered.unwrap_err(),
        ClientError::UnexpectedResponse(r#"{"error":{}}"#.to_owned())
    );
}

type Requests = Arc<Mutex<Vec<(String, Value)>>>;

/// A server that records every request's path and JSON body, and answers as
/// the real one would for a 004 account.
async fn recording_server() -> (ServerClient, Requests) {
    let requests = Requests::default();
    let app = Router::new()
        .fallback(any(record))
        .with_state(requests.clone());
    (ServerClient::new(&serve(app).await), requests)
}

async fn record(
    State(requests): State<Requests>,
    uri: axum::http::Uri,
    Json(body): Json<Value>,
) -> Json<Value> {
    requests.lock().unwrap().push((uri.path().to_owned(), body));
    Json(json!({
        "identifier": EMAIL,
        "pw_nonce": SEED,
        "version": "004",
        "session": {
            "access_token": "1:a:b",
            "refresh_token": "1:a:c",
            "access_expiration": 1,
            "refresh_expiration": 2,
        },
    }))
}

#[tokio::test]
async fn registers_with_every_field_standard_notes_sends() {
    let (client, requests) = recording_server().await;

    client
        .register(&key_params(EMAIL), &SERVER_PASSWORD)
        .await
        .unwrap();

    assert_eq!(
        requests.lock().unwrap()[0],
        (
            "/v1/users".to_owned(),
            json!({
                "email": EMAIL,
                "password": "07".repeat(32),
                "pw_nonce": SEED,
                "version": "004",
                "origination": "registration",
            })
        )
    );
}

#[tokio::test]
async fn signs_in_with_a_256_bit_code_verifier_in_hex() {
    let (client, requests) = recording_server().await;
    let challenge = client.key_params(EMAIL).await.unwrap();

    client.sign_in(challenge, &SERVER_PASSWORD).await.unwrap();

    let (path, body) = requests.lock().unwrap()[1].clone();
    assert_eq!(path, "/v2/login");
    let verifier = body["code_verifier"].as_str().unwrap();
    assert_eq!(verifier.len(), 64, "{verifier}");
    assert!(
        verifier
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    );
}

#[tokio::test]
async fn accepts_a_server_address_ending_in_a_slash() {
    let (client, _database) = real_server_at(|url| format!("{url}/")).await;

    let registered = client.register(&key_params(EMAIL), &SERVER_PASSWORD).await;

    assert!(registered.is_ok(), "{registered:?}");
}

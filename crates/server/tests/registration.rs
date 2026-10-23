//! Registering an account through the HTTP API, as the Standard Notes client
//! does with `POST /v1/users`, against a real server and database.

mod support;

use serde_json::{Value, json};
use support::{EMAIL, NONCE, NOW, SERVER_PASSWORD, TestServer, file_contains, registration, with};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

fn error(message: &str) -> Value {
    json!({ "error": { "message": message } })
}

#[tokio::test]
async fn registers_an_account_and_returns_its_key_params() {
    let server = TestServer::start().await;

    let (status, body) = server.register(registration()).await;

    assert_eq!(status, 200);
    assert_eq!(
        body["key_params"],
        json!({
            "identifier": EMAIL,
            "pw_nonce": NONCE,
            "version": "004",
            "origination": "registration",
            "created": "1792756800000",
        })
    );
    assert_eq!(body["user"]["email"], EMAIL);
}

#[tokio::test]
async fn gives_each_account_a_uuid() {
    let server = TestServer::start().await;

    let (_, first) = server.register(registration()).await;
    let (_, second) = server
        .register(with("email", json!("someone@example.com")))
        .await;

    let first = first["user"]["uuid"].as_str().unwrap();
    let second = second["user"]["uuid"].as_str().unwrap();
    assert!(uuid_shaped(first), "{first}");
    assert_ne!(first, second);
}

#[tokio::test]
async fn starts_a_session_that_lasts_sixty_days_with_a_year_to_refresh_it() {
    let server = TestServer::start().await;

    let (_, body) = server.register(registration()).await;

    let session = &body["session"];
    assert_eq!(session["access_expiration"], NOW + 60 * DAY_MS);
    assert_eq!(session["refresh_expiration"], NOW + 365 * DAY_MS);
    assert_eq!(session["readonly_access"], false);
}

#[tokio::test]
async fn gives_session_tokens_in_standard_notes_format() {
    let server = TestServer::start().await;

    let (_, body) = server.register(registration()).await;

    let access = body["session"]["access_token"].as_str().unwrap();
    let refresh = body["session"]["refresh_token"].as_str().unwrap();
    let [version, session, token] = parts(access);
    assert_eq!(version, "1");
    assert!(uuid_shaped(session), "{session}");
    assert!(token_shaped(token), "{token}");
    let [_, refresh_session, refresh_token] = parts(refresh);
    assert_eq!(refresh_session, session);
    assert!(token_shaped(refresh_token), "{refresh_token}");
    assert_ne!(refresh_token, token);
}

#[tokio::test]
async fn stores_the_email_trimmed_and_in_lower_case() {
    let server = TestServer::start().await;

    let (_, body) = server
        .register(with("email", json!("  David@Example.COM ")))
        .await;

    assert_eq!(body["key_params"]["identifier"], EMAIL);
    assert_eq!(body["user"]["email"], EMAIL);
}

#[tokio::test]
async fn leaves_origination_and_created_out_when_they_are_not_sent() {
    let server = TestServer::start().await;
    let mut body = registration();
    body.as_object_mut().unwrap().remove("origination");
    body.as_object_mut().unwrap().remove("created");

    let (status, body) = server.register(body).await;

    assert_eq!(status, 200);
    assert_eq!(
        body["key_params"],
        json!({ "identifier": EMAIL, "pw_nonce": NONCE, "version": "004" })
    );
}

#[tokio::test]
async fn refuses_an_email_that_is_already_registered() {
    let server = TestServer::start().await;
    server.register(registration()).await;

    let (status, body) = server
        .register(with("email", json!("DAVID@example.com")))
        .await;

    assert_eq!(status, 400);
    assert_eq!(body, error("This email is already registered."));
}

#[tokio::test]
async fn refuses_a_missing_email_or_password() {
    let server = TestServer::start().await;

    for field in ["email", "password"] {
        let mut body = registration();
        body.as_object_mut().unwrap().remove(field);

        let (status, body) = server.register(body).await;

        assert_eq!(status, 400, "{field}");
        assert_eq!(
            body,
            error("Please enter an email and a password to register."),
            "{field}"
        );
    }
}

#[tokio::test]
async fn refuses_an_invalid_email() {
    let server = TestServer::start().await;

    let (status, body) = server.register(with("email", json!("david"))).await;

    assert_eq!(status, 400);
    assert_eq!(body, error("Enter a valid email address."));
}

#[tokio::test]
async fn refuses_a_protocol_version_other_than_004() {
    let server = TestServer::start().await;

    for version in [json!("003"), json!("005"), Value::Null] {
        let (status, body) = server.register(with("version", version.clone())).await;

        assert_eq!(status, 400, "{version}");
        assert_eq!(
            body,
            error("Please update randomtask to register."),
            "{version}"
        );
    }
}

#[tokio::test]
async fn refuses_a_seed_that_is_not_256_bits_of_hex() {
    let server = TestServer::start().await;

    for nonce in [
        json!("abc"),
        json!("Z".repeat(64)),
        json!(NONCE.to_uppercase()),
        Value::Null,
    ] {
        let (status, body) = server.register(with("pw_nonce", nonce.clone())).await;

        assert_eq!(status, 400, "{nonce}");
        assert_eq!(body, error("The key parameters are invalid."), "{nonce}");
    }
}

#[tokio::test]
async fn refuses_a_body_that_is_not_json() {
    let server = TestServer::start().await;

    let response = server
        .client
        .post(format!("{}/v1/users", server.url))
        .header("content-type", "application/json")
        .body("not json")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
    assert_eq!(
        response.json::<Value>().await.unwrap(),
        error("The request is not valid JSON.")
    );
}

#[tokio::test]
async fn stores_neither_the_server_password_nor_the_session_tokens() {
    let server = TestServer::start().await;

    let (_, body) = server.register(registration()).await;

    let path = server.database_path();
    assert!(!file_contains(&path, SERVER_PASSWORD));
    for token in ["access_token", "refresh_token"] {
        let [_, _, token] = parts(body["session"][token].as_str().unwrap());
        assert!(!file_contains(&path, token), "{token}");
    }
}

#[tokio::test]
async fn reports_a_storage_failure_without_revealing_it() {
    for table in ["sessions", "users"] {
        let server = TestServer::start().await;
        rusqlite::Connection::open(server.database_path())
            .unwrap()
            .execute_batch(&format!("DROP TABLE {table};"))
            .unwrap();

        let (status, body) = server.register(registration()).await;

        assert_eq!(status, 500, "{table}");
        assert_eq!(
            body,
            error("Something went wrong. Please try again."),
            "{table}"
        );
    }
}

#[tokio::test]
async fn keeps_no_account_when_its_session_cannot_be_stored() {
    let server = TestServer::start().await;
    rusqlite::Connection::open(server.database_path())
        .unwrap()
        .execute_batch("DROP TABLE sessions;")
        .unwrap();
    server.register(registration()).await;
    // Opening the database again creates the missing table.
    randomtask_server::Database::open(&server.database_path()).unwrap();

    let (status, _) = server.register(registration()).await;

    assert_eq!(status, 200);
}

fn parts(token: &str) -> [&str; 3] {
    let parts: Vec<&str> = token.split(':').collect();
    parts.try_into().unwrap()
}

fn uuid_shaped(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok()
}

/// 48 random bytes in unpadded base64url.
fn token_shaped(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

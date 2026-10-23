//! Signing in through the HTTP API as the Standard Notes client does: it asks
//! for the key parameters with a PKCE code challenge (`POST /v2/login-params`),
//! derives the server password, then signs in with it and the code verifier
//! (`POST /v2/login`). Against a real server and database.

mod support;

use serde_json::{Value, json};
use support::{EMAIL, NONCE, NOW, SERVER_PASSWORD, TestServer, file_contains, registration};

const HOUR_MS: i64 = 60 * 60 * 1000;
const DAY_MS: i64 = 24 * HOUR_MS;
/// A code verifier as the client makes it: 256 random bits as hex.
const VERIFIER: &str = "5f1d2c3b4a59687706f5e4d3c2b1a0998877665544332211ffeeddccbbaa0099";
/// base64url, without padding, of the hex SHA-256 of `VERIFIER`, computed
/// independently with Python's hashlib and base64.
const CHALLENGE: &str =
    "YjcwODBhOGViNzg2ODJiNTc2MDMwMjliNzM3ZjU0MDhiZWFkNjhjNTYxNmVkNjU2Mjc5YjM4MTQyODM2NmJmMQ";

fn error(message: &str) -> Value {
    json!({ "error": { "message": message } })
}

fn wrong_credentials() -> Value {
    error("Invalid email or password")
}

async fn registered() -> TestServer {
    let server = TestServer::start().await;
    server.register(registration()).await;
    server
}

async fn login_params(server: &TestServer, email: &str) -> (u16, Value) {
    server
        .post(
            "/v2/login-params",
            json!({ "email": email, "code_challenge": CHALLENGE }),
        )
        .await
}

async fn login(server: &TestServer, password: &str, verifier: &str) -> (u16, Value) {
    server
        .post(
            "/v2/login",
            json!({ "email": EMAIL, "password": password, "code_verifier": verifier }),
        )
        .await
}

#[tokio::test]
async fn gives_the_key_params_of_a_registered_account() {
    let server = registered().await;

    let (status, body) = login_params(&server, EMAIL).await;

    assert_eq!(status, 200);
    assert_eq!(
        body,
        json!({ "identifier": EMAIL, "pw_nonce": NONCE, "version": "004" })
    );
}

#[tokio::test]
async fn finds_the_account_whatever_the_case_and_spacing_of_the_email() {
    let server = registered().await;

    let (_, body) = login_params(&server, " David@Example.COM ").await;

    assert_eq!(body["pw_nonce"], NONCE);
}

#[tokio::test]
async fn makes_up_key_params_for_an_unknown_email_so_accounts_cannot_be_discovered() {
    let server = registered().await;

    let (status, body) = login_params(&server, "nobody@example.com").await;

    assert_eq!(status, 200);
    // The hex SHA-256 of the email followed by the secret, from Python's hashlib.
    assert_eq!(
        body,
        json!({
            "identifier": "nobody@example.com",
            "pw_nonce": "6fb39d6cb3dcc9ed89c21daebbcaa122e8a1c60164d2a8c9cbfa8929fff3bd24",
            "version": "004",
        })
    );
}

#[tokio::test]
async fn refuses_key_params_without_a_code_challenge_or_email() {
    let server = registered().await;

    let (status, body) = server
        .post("/v2/login-params", json!({ "email": EMAIL }))
        .await;
    assert_eq!(status, 400);
    assert_eq!(body, error("Please provide the code challenge parameter."));

    let (status, body) = server
        .post("/v2/login-params", json!({ "code_challenge": CHALLENGE }))
        .await;
    assert_eq!(status, 400);
    assert_eq!(body, error("Enter your email address."));
}

#[tokio::test]
async fn signs_in_with_the_server_password_and_code_verifier() {
    let server = registered().await;
    login_params(&server, EMAIL).await;

    let (status, body) = login(&server, SERVER_PASSWORD, VERIFIER).await;

    assert_eq!(status, 200);
    assert_eq!(body["user"]["email"], EMAIL);
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
    assert_eq!(body["session"]["access_expiration"], NOW + 60 * DAY_MS);
    assert_eq!(body["session"]["refresh_expiration"], NOW + 365 * DAY_MS);
}

#[tokio::test]
async fn returns_the_same_user_as_registration() {
    let server = TestServer::start().await;
    let (_, registered) = server.register(registration()).await;
    login_params(&server, EMAIL).await;

    let (_, signed_in) = login(&server, SERVER_PASSWORD, VERIFIER).await;

    assert_eq!(signed_in["user"]["uuid"], registered["user"]["uuid"]);
}

#[tokio::test]
async fn starts_a_new_session_at_each_sign_in() {
    let server = registered().await;
    login_params(&server, EMAIL).await;
    let (_, first) = login(&server, SERVER_PASSWORD, VERIFIER).await;
    login_params(&server, EMAIL).await;

    let (_, second) = login(&server, SERVER_PASSWORD, VERIFIER).await;

    assert_ne!(
        first["session"]["access_token"],
        second["session"]["access_token"]
    );
    let path = server.database_path();
    for token in [&first, &second].map(|body| body["session"]["access_token"].as_str().unwrap()) {
        assert!(!file_contains(&path, token.rsplit(':').next().unwrap()));
    }
}

#[tokio::test]
async fn refuses_a_wrong_server_password() {
    let server = registered().await;
    login_params(&server, EMAIL).await;

    let (status, body) = login(&server, &"0".repeat(64), VERIFIER).await;

    assert_eq!(status, 401);
    assert_eq!(body, wrong_credentials());
}

#[tokio::test]
async fn refuses_an_unknown_email_in_the_same_words() {
    let server = registered().await;
    login_params(&server, "nobody@example.com").await;

    let (status, body) = server
        .post(
            "/v2/login",
            json!({
                "email": "nobody@example.com",
                "password": SERVER_PASSWORD,
                "code_verifier": VERIFIER,
            }),
        )
        .await;

    assert_eq!(status, 401);
    assert_eq!(body, wrong_credentials());
}

#[tokio::test]
async fn refuses_a_code_verifier_that_does_not_match_the_challenge() {
    let server = registered().await;
    login_params(&server, EMAIL).await;

    let (status, body) = login(&server, SERVER_PASSWORD, &"1".repeat(64)).await;

    assert_eq!(status, 401);
    assert_eq!(body, wrong_credentials());
}

#[tokio::test]
async fn refuses_a_sign_in_without_asking_for_key_params_first() {
    let server = registered().await;

    let (status, body) = login(&server, SERVER_PASSWORD, VERIFIER).await;

    assert_eq!(status, 401);
    assert_eq!(body, wrong_credentials());
}

#[tokio::test]
async fn accepts_each_code_challenge_only_once() {
    let server = registered().await;
    login_params(&server, EMAIL).await;
    login(&server, SERVER_PASSWORD, VERIFIER).await;

    let (status, body) = login(&server, SERVER_PASSWORD, VERIFIER).await;

    assert_eq!(status, 401);
    assert_eq!(body, wrong_credentials());
}

#[tokio::test]
async fn uses_up_the_code_challenge_even_when_the_password_is_wrong() {
    let server = registered().await;
    login_params(&server, EMAIL).await;
    login(&server, &"0".repeat(64), VERIFIER).await;

    let (status, _) = login(&server, SERVER_PASSWORD, VERIFIER).await;

    assert_eq!(status, 401);
}

#[tokio::test]
async fn accepts_a_code_challenge_for_an_hour() {
    let server = registered().await;
    login_params(&server, EMAIL).await;
    server.advance(HOUR_MS - 1);

    let (status, _) = login(&server, SERVER_PASSWORD, VERIFIER).await;

    assert_eq!(status, 200);
}

#[tokio::test]
async fn refuses_a_code_challenge_after_an_hour() {
    let server = registered().await;
    login_params(&server, EMAIL).await;
    server.advance(HOUR_MS);

    let (status, body) = login(&server, SERVER_PASSWORD, VERIFIER).await;

    assert_eq!(status, 401);
    assert_eq!(body, wrong_credentials());
}

#[tokio::test]
async fn refuses_a_sign_in_with_missing_fields() {
    let server = registered().await;

    for field in ["email", "password", "code_verifier"] {
        let mut body =
            json!({ "email": EMAIL, "password": SERVER_PASSWORD, "code_verifier": VERIFIER });
        body.as_object_mut().unwrap().remove(field);

        let (status, body) = server.post("/v2/login", body).await;

        assert_eq!(status, 401, "{field}");
        assert_eq!(
            body,
            json!({ "error": { "tag": "invalid-auth", "message": "Invalid login credentials." } }),
            "{field}"
        );
    }
}

#[tokio::test]
async fn refuses_bodies_that_are_not_json() {
    let server = registered().await;

    for path in ["/v2/login-params", "/v2/login"] {
        let response = server
            .client
            .post(format!("{}{path}", server.url))
            .header("content-type", "application/json")
            .body("not json")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status().as_u16(), 400, "{path}");
        assert_eq!(
            response.json::<Value>().await.unwrap(),
            error("The request is not valid JSON."),
            "{path}"
        );
    }
}

#[tokio::test]
async fn reports_storage_failures_without_revealing_them() {
    for table in ["code_challenges", "sessions"] {
        let server = registered().await;
        login_params(&server, EMAIL).await;
        rusqlite::Connection::open(server.database_path())
            .unwrap()
            .execute_batch(&format!("PRAGMA foreign_keys = OFF; DROP TABLE {table};"))
            .unwrap();

        let (status, body) = login(&server, SERVER_PASSWORD, VERIFIER).await;

        assert_eq!(status, 500, "{table}");
        assert_eq!(
            body,
            error("Something went wrong. Please try again."),
            "{table}"
        );
    }
}

#[tokio::test]
async fn reports_a_storage_failure_when_giving_key_params() {
    for table in ["code_challenges", "users"] {
        let server = registered().await;
        rusqlite::Connection::open(server.database_path())
            .unwrap()
            .execute_batch(&format!("PRAGMA foreign_keys = OFF; DROP TABLE {table};"))
            .unwrap();

        let (status, body) = login_params(&server, EMAIL).await;

        assert_eq!(status, 500, "{table}");
        assert_eq!(
            body,
            error("Something went wrong. Please try again."),
            "{table}"
        );
    }
}

#[tokio::test]
async fn refuses_key_params_for_an_invalid_email() {
    let server = registered().await;

    let (status, body) = login_params(&server, "david").await;

    assert_eq!(status, 400);
    assert_eq!(body, error("Enter a valid email address."));
}

#[tokio::test]
async fn refuses_a_sign_in_with_an_invalid_email_as_wrong_credentials() {
    let server = registered().await;

    let (status, body) = server
        .post(
            "/v2/login",
            json!({ "email": "david", "password": SERVER_PASSWORD, "code_verifier": VERIFIER }),
        )
        .await;

    assert_eq!(status, 401);
    assert_eq!(body, wrong_credentials());
}

#[tokio::test]
async fn restarts_the_hour_when_the_same_code_challenge_is_sent_again() {
    let server = registered().await;
    login_params(&server, EMAIL).await;
    server.advance(HOUR_MS);
    login_params(&server, EMAIL).await;

    let (status, _) = login(&server, SERVER_PASSWORD, VERIFIER).await;

    assert_eq!(status, 200);
}

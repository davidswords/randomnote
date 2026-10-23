//! Locking sign-in after repeated failures, as Standard Notes does: six
//! failed sign-ins in a row lock the email for an hour. Against a real server
//! and database.

mod support;

use serde_json::{Value, json};
use support::{EMAIL, SERVER_PASSWORD, TestServer, registration};

const HOUR_MS: i64 = 60 * 60 * 1000;
const VERIFIER: &str = "5f1d2c3b4a59687706f5e4d3c2b1a0998877665544332211ffeeddccbbaa0099";
const CHALLENGE: &str =
    "YjcwODBhOGViNzg2ODJiNTc2MDMwMjliNzM3ZjU0MDhiZWFkNjhjNTYxNmVkNjU2Mjc5YjM4MTQyODM2NmJmMQ";
const WRONG_PASSWORD: &str = "0000000000000000000000000000000000000000000000000000000000000000";

async fn registered() -> TestServer {
    let server = TestServer::start().await;
    server.register(registration()).await;
    server
}

async fn sign_in_as(server: &TestServer, email: &str, password: &str) -> (u16, Value) {
    server
        .post(
            "/v2/login-params",
            json!({ "email": email, "code_challenge": CHALLENGE }),
        )
        .await;
    server
        .post(
            "/v2/login",
            json!({ "email": email, "password": password, "code_verifier": VERIFIER }),
        )
        .await
}

async fn sign_in(server: &TestServer, password: &str) -> (u16, Value) {
    sign_in_as(server, EMAIL, password).await
}

async fn fail(server: &TestServer, email: &str, times: usize) {
    for _ in 0..times {
        let (status, _) = sign_in_as(server, email, WRONG_PASSWORD).await;
        assert_eq!(status, 401);
    }
}

fn locked() -> Value {
    json!({ "error": { "message": "Too many failed sign-ins. Please try again in an hour." } })
}

#[tokio::test]
async fn still_signs_in_after_five_failures() {
    let server = registered().await;
    fail(&server, EMAIL, 5).await;

    let (status, _) = sign_in(&server, SERVER_PASSWORD).await;

    assert_eq!(status, 200);
}

#[tokio::test]
async fn locks_sign_in_after_six_failures_even_with_the_right_password() {
    let server = registered().await;
    fail(&server, EMAIL, 6).await;

    let (status, body) = sign_in(&server, SERVER_PASSWORD).await;

    assert_eq!(status, 423);
    assert_eq!(body, locked());
}

#[tokio::test]
async fn finds_the_lock_whatever_the_case_and_spacing_of_the_email() {
    let server = registered().await;
    fail(&server, EMAIL, 6).await;

    let (status, _) = sign_in_as(&server, " DAVID@example.com ", SERVER_PASSWORD).await;

    assert_eq!(status, 423);
}

#[tokio::test]
async fn starts_counting_again_after_a_successful_sign_in() {
    let server = registered().await;
    fail(&server, EMAIL, 5).await;
    sign_in(&server, SERVER_PASSWORD).await;
    fail(&server, EMAIL, 5).await;

    let (status, _) = sign_in(&server, SERVER_PASSWORD).await;

    assert_eq!(status, 200);
}

#[tokio::test]
async fn keeps_the_lock_for_an_hour() {
    let server = registered().await;
    fail(&server, EMAIL, 6).await;
    server.advance(HOUR_MS - 1);

    let (status, _) = sign_in(&server, SERVER_PASSWORD).await;

    assert_eq!(status, 423);
}

#[tokio::test]
async fn lifts_the_lock_after_an_hour() {
    let server = registered().await;
    fail(&server, EMAIL, 6).await;
    server.advance(HOUR_MS);

    let (status, _) = sign_in(&server, SERVER_PASSWORD).await;

    assert_eq!(status, 200);
}

#[tokio::test]
async fn gives_six_more_tries_once_the_lock_is_lifted() {
    let server = registered().await;
    fail(&server, EMAIL, 6).await;
    server.advance(HOUR_MS);
    fail(&server, EMAIL, 5).await;

    let (status, _) = sign_in(&server, SERVER_PASSWORD).await;

    assert_eq!(status, 200);
}

#[tokio::test]
async fn locks_only_the_email_that_failed() {
    let server = registered().await;
    let other = json!("someone@example.com");
    let mut body = registration();
    body["email"] = other;
    server.register(body).await;
    fail(&server, "someone@example.com", 6).await;

    let (status, _) = sign_in(&server, SERVER_PASSWORD).await;

    assert_eq!(status, 200);
}

#[tokio::test]
async fn locks_an_unknown_email_the_same_way_so_accounts_cannot_be_discovered() {
    let server = registered().await;
    fail(&server, "nobody@example.com", 6).await;

    let (status, body) = sign_in_as(&server, "nobody@example.com", WRONG_PASSWORD).await;

    assert_eq!(status, 423);
    assert_eq!(body, locked());
}

#[tokio::test]
async fn reports_a_storage_failure_when_counting_failures() {
    let server = registered().await;
    rusqlite::Connection::open(server.database_path())
        .unwrap()
        .execute_batch("DROP TABLE failed_sign_ins;")
        .unwrap();

    let (status, body) = sign_in(&server, WRONG_PASSWORD).await;

    assert_eq!(status, 500);
    assert_eq!(
        body,
        json!({ "error": { "message": "Something went wrong. Please try again." } })
    );
}

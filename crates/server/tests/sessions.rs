//! Refreshing a session and signing out, as the Standard Notes client does
//! with `POST /v1/sessions/refresh` and `POST /v1/logout`. Against a real
//! server and database.

mod support;

use serde_json::{Value, json};
use support::{NOW, TestServer, file_contains, registration};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

struct Tokens {
    access: String,
    refresh: String,
}

async fn signed_up() -> (TestServer, Tokens) {
    let server = TestServer::start().await;
    let (_, body) = server.register(registration()).await;
    let tokens = tokens(&body["session"]);
    (server, tokens)
}

fn tokens(session: &Value) -> Tokens {
    Tokens {
        access: session["access_token"].as_str().unwrap().to_owned(),
        refresh: session["refresh_token"].as_str().unwrap().to_owned(),
    }
}

async fn refresh(server: &TestServer, tokens: &Tokens) -> (u16, Value) {
    server
        .post(
            "/v1/sessions/refresh",
            json!({ "access_token": tokens.access, "refresh_token": tokens.refresh }),
        )
        .await
}

async fn sign_out(server: &TestServer, authorization: Option<&str>) -> (u16, String) {
    let mut request = server.client.post(format!("{}/v1/logout", server.url));
    if let Some(authorization) = authorization {
        request = request.header("authorization", authorization);
    }
    let response = request.send().await.unwrap();
    (response.status().as_u16(), response.text().await.unwrap())
}

fn bearer(tokens: &Tokens) -> String {
    format!("Bearer {}", tokens.access)
}

fn error(tag: &str, message: &str) -> Value {
    json!({ "error": { "tag": tag, "message": message } })
}

fn invalid_refresh_token() -> Value {
    error("invalid-refresh-token", "The refresh token is not valid.")
}

fn invalid_session_token() -> String {
    json!({ "error": { "message": "Invalid session token." } }).to_string()
}

fn with_secret(token: &str, secret: &str) -> String {
    let (prefix, _) = token.rsplit_once(':').unwrap();
    format!("{prefix}:{secret}")
}

#[tokio::test]
async fn refreshes_a_session_with_new_tokens_and_expirations() {
    let (server, tokens) = signed_up().await;
    server.advance(DAY_MS);

    let (status, body) = refresh(&server, &tokens).await;

    assert_eq!(status, 200);
    let session = &body["session"];
    assert_eq!(session["access_expiration"], NOW + DAY_MS + 60 * DAY_MS);
    assert_eq!(session["refresh_expiration"], NOW + DAY_MS + 365 * DAY_MS);
    assert_eq!(session["readonly_access"], false);
    let new = self::tokens(session);
    assert_ne!(new.access, tokens.access);
    assert_ne!(new.refresh, tokens.refresh);
}

#[tokio::test]
async fn keeps_the_same_session_when_refreshing() {
    let (server, tokens) = signed_up().await;

    let (_, body) = refresh(&server, &tokens).await;

    let new = self::tokens(&body["session"]);
    let session = |token: &str| token.split(':').nth(1).unwrap().to_owned();
    assert_eq!(session(&new.access), session(&tokens.access));
    assert_eq!(session(&new.refresh), session(&tokens.refresh));
}

#[tokio::test]
async fn refreshes_again_with_the_new_tokens() {
    let (server, tokens) = signed_up().await;
    let (_, body) = refresh(&server, &tokens).await;

    let (status, _) = refresh(&server, &self::tokens(&body["session"])).await;

    assert_eq!(status, 200);
}

#[tokio::test]
async fn refuses_the_old_refresh_token_once_it_has_been_used() {
    let (server, tokens) = signed_up().await;
    refresh(&server, &tokens).await;

    let (status, body) = refresh(&server, &tokens).await;

    assert_eq!(status, 400);
    assert_eq!(body, invalid_refresh_token());
}

#[tokio::test]
async fn refuses_a_refresh_token_with_the_wrong_secret() {
    let (server, tokens) = signed_up().await;
    let forged = Tokens {
        refresh: with_secret(&tokens.refresh, &"A".repeat(64)),
        ..tokens
    };

    let (status, body) = refresh(&server, &forged).await;

    assert_eq!(status, 400);
    assert_eq!(body, invalid_refresh_token());
}

#[tokio::test]
async fn refuses_an_access_token_from_another_session() {
    let (server, tokens) = signed_up().await;
    let mut other = registration();
    other["email"] = json!("someone@example.com");
    let (_, body) = server.register(other).await;
    let mixed = Tokens {
        access: self::tokens(&body["session"]).access,
        ..tokens
    };

    let (status, body) = refresh(&server, &mixed).await;

    assert_eq!(status, 400);
    assert_eq!(body, invalid_refresh_token());
}

#[tokio::test]
async fn refuses_tokens_that_are_not_in_session_token_format() {
    let (server, tokens) = signed_up().await;

    for refresh_token in ["", "abc", "1:not-a-session:abc", "2:x:y:z"] {
        let malformed = Tokens {
            refresh: refresh_token.to_owned(),
            access: tokens.access.clone(),
        };

        let (status, body) = refresh(&server, &malformed).await;

        assert_eq!(status, 400, "{refresh_token}");
        assert_eq!(body, invalid_refresh_token(), "{refresh_token}");
    }
}

#[tokio::test]
async fn refuses_a_real_refresh_token_with_another_version_or_extra_parts() {
    let (server, tokens) = signed_up().await;

    for refresh_token in [
        format!("2{}", &tokens.refresh[1..]),
        format!("{}:extra", tokens.refresh),
    ] {
        let altered = Tokens {
            refresh: refresh_token.clone(),
            access: tokens.access.clone(),
        };

        let (status, body) = refresh(&server, &altered).await;

        assert_eq!(status, 400, "{refresh_token}");
        assert_eq!(body, invalid_refresh_token(), "{refresh_token}");
    }
}

#[tokio::test]
async fn leaves_other_sessions_alone_when_refreshing() {
    let (server, first) = signed_up().await;
    let mut other = registration();
    other["email"] = json!("someone@example.com");
    let (_, body) = server.register(other).await;
    let second = tokens(&body["session"]);

    refresh(&server, &first).await;

    let (status, _) = refresh(&server, &second).await;
    assert_eq!(status, 200);
}

#[tokio::test]
async fn refuses_an_access_token_that_is_not_in_session_token_format() {
    let (server, tokens) = signed_up().await;
    let malformed = Tokens {
        access: "abc".to_owned(),
        ..tokens
    };

    let (status, body) = refresh(&server, &malformed).await;

    assert_eq!(status, 400);
    assert_eq!(body, invalid_refresh_token());
}

#[tokio::test]
async fn refreshes_until_the_refresh_token_expires_after_a_year() {
    let (server, tokens) = signed_up().await;
    server.advance(365 * DAY_MS - 1);

    let (status, _) = refresh(&server, &tokens).await;

    assert_eq!(status, 200);
}

#[tokio::test]
async fn refuses_an_expired_refresh_token() {
    let (server, tokens) = signed_up().await;
    server.advance(365 * DAY_MS);

    let (status, body) = refresh(&server, &tokens).await;

    assert_eq!(status, 400);
    assert_eq!(
        body,
        error("expired-refresh-token", "The refresh token has expired.")
    );
}

#[tokio::test]
async fn refuses_a_refresh_without_both_tokens() {
    let (server, tokens) = signed_up().await;

    for body in [
        json!({ "access_token": tokens.access }),
        json!({ "refresh_token": tokens.refresh }),
    ] {
        let (status, body) = server.post("/v1/sessions/refresh", body).await;

        assert_eq!(status, 400);
        assert_eq!(
            body,
            json!({ "error": { "message": "Please provide all required parameters." } })
        );
    }
}

#[tokio::test]
async fn stores_no_refreshed_token() {
    let (server, tokens) = signed_up().await;

    let (_, body) = refresh(&server, &tokens).await;

    let new = self::tokens(&body["session"]);
    for token in [new.access, new.refresh] {
        let secret = token.rsplit(':').next().unwrap().to_owned();
        assert!(!file_contains(&server.database_path(), &secret));
    }
}

#[tokio::test]
async fn signs_out_and_ends_the_session() {
    let (server, tokens) = signed_up().await;

    let (status, body) = sign_out(&server, Some(&bearer(&tokens))).await;

    assert_eq!(status, 204);
    assert_eq!(body, "");
    let (status, body) = refresh(&server, &tokens).await;
    assert_eq!(status, 400);
    assert_eq!(body, invalid_refresh_token());
}

#[tokio::test]
async fn ends_only_the_session_signed_out_of() {
    let (server, first) = signed_up().await;
    let (_, body) = refresh(&server, &first).await;
    let first = tokens(&body["session"]);
    let mut again = registration();
    again["email"] = json!("someone@example.com");
    let (_, body) = server.register(again).await;
    let second = tokens(&body["session"]);

    sign_out(&server, Some(&bearer(&second))).await;

    let (status, _) = refresh(&server, &first).await;
    assert_eq!(status, 200);
}

#[tokio::test]
async fn refuses_to_sign_out_with_an_invalid_access_token() {
    let (server, tokens) = signed_up().await;
    let forged = with_secret(&tokens.access, &"A".repeat(64));

    for authorization in [
        None,
        Some("Basic abc".to_owned()),
        Some(format!("Bearer {forged}")),
        Some(format!("Bearer {}", tokens.refresh)),
        Some("Bearer nonsense".to_owned()),
    ] {
        let (status, body) = sign_out(&server, authorization.as_deref()).await;

        assert_eq!(status, 401, "{authorization:?}");
        assert_eq!(body, invalid_session_token(), "{authorization:?}");
    }
    let (status, _) = refresh(&server, &tokens).await;
    assert_eq!(status, 200);
}

#[tokio::test]
async fn refuses_to_sign_out_twice() {
    let (server, tokens) = signed_up().await;
    sign_out(&server, Some(&bearer(&tokens))).await;

    let (status, body) = sign_out(&server, Some(&bearer(&tokens))).await;

    assert_eq!(status, 401);
    assert_eq!(body, invalid_session_token());
}

#[tokio::test]
async fn reports_storage_failures_without_revealing_them() {
    let (server, tokens) = signed_up().await;
    rusqlite::Connection::open(server.database_path())
        .unwrap()
        .execute_batch("DROP TABLE sessions;")
        .unwrap();
    let failure = json!({ "error": { "message": "Something went wrong. Please try again." } });

    let (status, body) = refresh(&server, &tokens).await;
    assert_eq!(status, 500);
    assert_eq!(body, failure);

    let (status, body) = sign_out(&server, Some(&bearer(&tokens))).await;
    assert_eq!(status, 500);
    assert_eq!(body, failure.to_string());
}

#[tokio::test]
async fn refuses_a_refresh_body_that_is_not_json() {
    let (server, _) = signed_up().await;

    let response = server
        .client
        .post(format!("{}/v1/sessions/refresh", server.url))
        .header("content-type", "application/json")
        .body("not json")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}

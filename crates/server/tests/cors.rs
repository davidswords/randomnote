//! Letting the web app call the server from another origin (CORS), as a
//! browser checks it: a preflight request first, then the real one. Against
//! a real server.

mod support;

use reqwest::header::HeaderMap;
use support::{TestServer, registration};

const WEB_APP: &str = "http://localhost:1421";

async fn preflight(server: &TestServer, origin: &str) -> (u16, HeaderMap) {
    let response = server
        .client
        .request(reqwest::Method::OPTIONS, format!("{}/v1/users", server.url))
        .header("origin", origin)
        .header("access-control-request-method", "POST")
        .header(
            "access-control-request-headers",
            "content-type, authorization",
        )
        .send()
        .await
        .unwrap();
    (response.status().as_u16(), response.headers().clone())
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).map(|value| value.to_str().unwrap())
}

#[tokio::test]
async fn allows_an_allowed_origin_to_post_json_with_a_token() {
    let server = TestServer::start_allowing(&[WEB_APP]).await;

    let (status, headers) = preflight(&server, WEB_APP).await;

    assert_eq!(status, 200);
    assert_eq!(
        header(&headers, "access-control-allow-origin"),
        Some(WEB_APP)
    );
    let methods = header(&headers, "access-control-allow-methods").unwrap();
    assert!(methods.contains("POST"), "{methods}");
    let allowed = header(&headers, "access-control-allow-headers").unwrap();
    assert!(allowed.contains("content-type"), "{allowed}");
    assert!(allowed.contains("authorization"), "{allowed}");
}

#[tokio::test]
async fn marks_responses_to_an_allowed_origin_as_readable() {
    let server = TestServer::start_allowing(&[WEB_APP]).await;

    let response = server
        .client
        .post(format!("{}/v1/users", server.url))
        .header("origin", WEB_APP)
        .json(&registration())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        header(response.headers(), "access-control-allow-origin"),
        Some(WEB_APP)
    );
}

#[tokio::test]
async fn allows_each_of_several_origins() {
    let server = TestServer::start_allowing(&[WEB_APP, "https://randomtask.ch"]).await;

    let (_, headers) = preflight(&server, "https://randomtask.ch").await;

    assert_eq!(
        header(&headers, "access-control-allow-origin"),
        Some("https://randomtask.ch")
    );
}

#[tokio::test]
async fn gives_no_permission_to_an_origin_that_is_not_allowed() {
    let server = TestServer::start_allowing(&[WEB_APP]).await;

    let (_, headers) = preflight(&server, "https://evil.example").await;

    assert_eq!(header(&headers, "access-control-allow-origin"), None);
}

#[tokio::test]
async fn gives_no_permission_to_any_origin_unless_some_are_allowed() {
    let server = TestServer::start().await;

    let (_, headers) = preflight(&server, WEB_APP).await;

    assert_eq!(header(&headers, "access-control-allow-origin"), None);
}

#[test]
fn refuses_an_origin_that_is_not_a_valid_header_value() {
    assert!(randomtask_server::cors(&["http://bad\norigin"]).is_err());
}

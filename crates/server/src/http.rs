//! The HTTP API, at the paths the Standard Notes client uses.

use std::sync::Arc;

use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::header::InvalidHeaderValue;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{Map, Value, json};
use tower_http::cors::CorsLayer;

use crate::accounts::{Accounts, SignedIn};
use crate::error::ServerError;
use crate::registration::{KeyParams, Registration};
use crate::session::{NewSession, Refresh};
use crate::sign_in::{KeyParamsRequest, SignIn};

/// Lets web pages at `origins`, such as the web app, call the server from the
/// browser. Without any, browsers keep pages elsewhere from reading responses.
pub fn cors(origins: &[&str]) -> Result<CorsLayer, InvalidHeaderValue> {
    let origins = origins
        .iter()
        .map(|origin| HeaderValue::from_str(origin))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::POST])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]))
}

pub fn router(accounts: Accounts) -> Router {
    Router::new()
        .route("/v1/users", post(register))
        .route("/v2/login-params", post(login_params))
        .route("/v2/login", post(login))
        .route("/v1/sessions/refresh", post(refresh))
        .route("/v1/logout", post(sign_out))
        .with_state(Arc::new(accounts))
}

async fn register(
    State(accounts): State<Arc<Accounts>>,
    body: Result<Json<Registration>, JsonRejection>,
) -> Result<Json<Value>, ServerError> {
    let Json(registration) = body.map_err(|_| ServerError::InvalidJson)?;
    let signed_in = off_async_threads(move || accounts.register(registration)).await?;
    Ok(Json(signed_in_body(&signed_in)))
}

async fn login_params(
    State(accounts): State<Arc<Accounts>>,
    body: Result<Json<KeyParamsRequest>, JsonRejection>,
) -> Result<Json<Value>, ServerError> {
    let Json(request) = body.map_err(|_| ServerError::InvalidJson)?;
    let params = off_async_threads(move || accounts.key_params(request)).await?;
    Ok(Json(key_params_body(&params)))
}

async fn login(
    State(accounts): State<Arc<Accounts>>,
    body: Result<Json<SignIn>, JsonRejection>,
) -> Result<Json<Value>, ServerError> {
    let Json(sign_in) = body.map_err(|_| ServerError::InvalidJson)?;
    let signed_in = off_async_threads(move || accounts.sign_in(sign_in)).await?;
    Ok(Json(signed_in_body(&signed_in)))
}

async fn refresh(
    State(accounts): State<Arc<Accounts>>,
    body: Result<Json<Refresh>, JsonRejection>,
) -> Result<Json<Value>, ServerError> {
    let Json(refresh) = body.map_err(|_| ServerError::InvalidJson)?;
    let session = off_async_threads(move || accounts.refresh(refresh)).await?;
    Ok(Json(json!({ "session": session_body(&session) })))
}

async fn sign_out(
    State(accounts): State<Arc<Accounts>>,
    headers: HeaderMap,
) -> Result<StatusCode, ServerError> {
    let access_token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(ServerError::InvalidSessionToken)?
        .to_owned();
    off_async_threads(move || accounts.sign_out(&access_token)).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// bcrypt and SQLite block, so they run off the async threads.
async fn off_async_threads<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, ServerError> + Send + 'static,
) -> Result<T, ServerError> {
    tokio::task::spawn_blocking(work)
        .await
        .expect("account work doesn't panic")
}

fn signed_in_body(signed_in: &SignedIn) -> Value {
    json!({
        "session": session_body(&signed_in.session),
        "key_params": key_params_body(&signed_in.key_params),
        "user": {
            "uuid": signed_in.user_uuid,
            "email": signed_in.key_params.identifier,
        },
    })
}

fn session_body(session: &NewSession) -> Value {
    json!({
        "access_token": session.full_access_token(),
        "refresh_token": session.full_refresh_token(),
        "access_expiration": session.access_expiration,
        "refresh_expiration": session.refresh_expiration,
        "readonly_access": false,
    })
}

fn key_params_body(params: &KeyParams) -> Value {
    let mut body = Map::new();
    body.insert("identifier".into(), json!(params.identifier));
    body.insert("pw_nonce".into(), json!(params.pw_nonce));
    body.insert("version".into(), json!(params.version));
    if let Some(origination) = &params.origination {
        body.insert("origination".into(), json!(origination));
    }
    if let Some(created) = &params.created {
        body.insert("created".into(), json!(created));
    }
    Value::Object(body)
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Storage(ref detail) => {
                eprintln!("storage error: {detail}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::IncompleteSignIn | Self::WrongCredentials | Self::InvalidSessionToken => {
                StatusCode::UNAUTHORIZED
            }
            Self::Locked => StatusCode::LOCKED,
            _ => StatusCode::BAD_REQUEST,
        };
        let tag = match self {
            Self::IncompleteSignIn => Some("invalid-auth"),
            Self::InvalidRefreshToken => Some("invalid-refresh-token"),
            Self::ExpiredRefreshToken => Some("expired-refresh-token"),
            _ => None,
        };
        let body = match tag {
            Some(tag) => json!({ "error": { "tag": tag, "message": self.to_string() } }),
            None => json!({ "error": { "message": self.to_string() } }),
        };
        (status, Json(body)).into_response()
    }
}

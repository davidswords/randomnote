//! The HTTP API, at the paths the Standard Notes client uses.

use std::sync::Arc;

use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{Map, Value, json};

use crate::accounts::{Accounts, SignedIn};
use crate::error::ServerError;
use crate::registration::{KeyParams, Registration};

pub fn router(accounts: Accounts) -> Router {
    Router::new()
        .route("/v1/users", post(register))
        .with_state(Arc::new(accounts))
}

async fn register(
    State(accounts): State<Arc<Accounts>>,
    body: Result<Json<Registration>, JsonRejection>,
) -> Result<Json<Value>, ServerError> {
    let Json(registration) = body.map_err(|_| ServerError::InvalidJson)?;
    // bcrypt and SQLite block, so they run off the async threads.
    let signed_in = tokio::task::spawn_blocking(move || accounts.register(registration))
        .await
        .expect("registration doesn't panic")?;
    Ok(Json(signed_in_body(&signed_in)))
}

fn signed_in_body(signed_in: &SignedIn) -> Value {
    let session = &signed_in.session;
    json!({
        "session": {
            "access_token": session.full_access_token(),
            "refresh_token": session.full_refresh_token(),
            "access_expiration": session.access_expiration,
            "refresh_expiration": session.refresh_expiration,
            "readonly_access": false,
        },
        "key_params": key_params_body(&signed_in.key_params),
        "user": {
            "uuid": signed_in.user_uuid,
            "email": signed_in.key_params.identifier,
        },
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
            _ => StatusCode::BAD_REQUEST,
        };
        let body = json!({ "error": { "message": self.to_string() } });
        (status, Json(body)).into_response()
    }
}

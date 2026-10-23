//! The app's client for the randomtask server. It speaks the Standard Notes
//! protocol: registering, then signing in with key parameters and PKCE.
//! It never sends the password, only the server password derived from it.

mod pkce;

use std::fmt;

use randomtask_crypto::KeyParams;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

const VERSION: &str = "004";

/// Why the server couldn't do what was asked, with a message for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    Unreachable,
    /// The server refused, with its own message for the user.
    Refused(String),
    /// The account uses a protocol version this app doesn't support. Never
    /// signing in with an older version stops a downgrade attack.
    UnsupportedVersion,
    UnexpectedResponse(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Unreachable => {
                "Can't reach the randomtask server. Check your connection and try again."
            }
            Self::Refused(message) => message,
            Self::UnsupportedVersion => {
                "This account needs a newer version of randomtask. Please update it."
            }
            Self::UnexpectedResponse(_) => {
                "The randomtask server sent a response the app doesn't understand."
            }
        })
    }
}

impl std::error::Error for ClientError {}

/// A signed-in session's tokens, and when they expire, in milliseconds since
/// the Unix epoch.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expiration: i64,
    pub refresh_expiration: i64,
}

/// The key parameters to derive the server password from, and the code
/// verifier that signing in with it must send.
#[derive(Debug)]
pub struct SignInChallenge {
    pub key_params: KeyParams,
    code_verifier: String,
}

pub struct ServerClient {
    url: String,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct SessionResponse {
    session: Session,
}

#[derive(Deserialize)]
struct KeyParamsResponse {
    identifier: String,
    pw_nonce: String,
    version: String,
}

impl ServerClient {
    /// A client for the server at `url`, such as `https://api.randomtask.ch`.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.trim_end_matches('/').to_owned(),
            http: reqwest::Client::new(),
        }
    }

    /// Registers the account and starts a session.
    pub async fn register(
        &self,
        params: &KeyParams,
        server_password: &[u8; 32],
    ) -> Result<Session, ClientError> {
        let body = json!({
            "email": params.identifier,
            "password": hex::encode(server_password),
            "pw_nonce": params.seed,
            "version": VERSION,
            "origination": "registration",
        });
        let response: SessionResponse = self.post("/v1/users", body).await?;
        Ok(response.session)
    }

    /// Asks for the account's key parameters, to derive the server password
    /// for signing in.
    pub async fn key_params(&self, email: &str) -> Result<SignInChallenge, ClientError> {
        let code_verifier = pkce::code_verifier();
        let body =
            json!({ "email": email, "code_challenge": pkce::code_challenge(&code_verifier) });
        let params: KeyParamsResponse = self.post("/v2/login-params", body).await?;
        if params.version != VERSION {
            return Err(ClientError::UnsupportedVersion);
        }
        Ok(SignInChallenge {
            key_params: KeyParams {
                identifier: params.identifier,
                seed: params.pw_nonce,
            },
            code_verifier,
        })
    }

    /// Signs in with the server password derived from the challenge's key
    /// parameters, and starts a session.
    pub async fn sign_in(
        &self,
        challenge: SignInChallenge,
        server_password: &[u8; 32],
    ) -> Result<Session, ClientError> {
        let body = json!({
            "email": challenge.key_params.identifier,
            "password": hex::encode(server_password),
            "code_verifier": challenge.code_verifier,
        });
        let response: SessionResponse = self.post("/v2/login", body).await?;
        Ok(response.session)
    }

    async fn post<T: DeserializeOwned>(&self, path: &str, body: Value) -> Result<T, ClientError> {
        let response = self
            .http
            .post(format!("{}{path}", self.url))
            .json(&body)
            .send()
            .await
            .map_err(|_| ClientError::Unreachable)?;
        let succeeded = response.status().is_success();
        let body: Value = response.json().await.map_err(unexpected)?;
        if succeeded {
            serde_json::from_value(body).map_err(unexpected)
        } else {
            Err(refusal(&body))
        }
    }
}

/// The server's message from an error body: `{"error": {"message": "..."}}`.
fn refusal(body: &Value) -> ClientError {
    match body["error"]["message"].as_str() {
        Some(message) => ClientError::Refused(message.to_owned()),
        None => ClientError::UnexpectedResponse(body.to_string()),
    }
}

fn unexpected(error: impl fmt::Display) -> ClientError {
    ClientError::UnexpectedResponse(error.to_string())
}

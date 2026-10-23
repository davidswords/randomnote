//! Checking a registration request before anything is stored.

use serde::Deserialize;

use crate::error::ServerError;

pub(crate) const VERSION: &str = "004";

/// A registration as the Standard Notes client sends it. `password` is the
/// server password, never the user's password.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct Registration {
    pub email: Option<String>,
    pub password: Option<String>,
    pub pw_nonce: Option<String>,
    pub version: Option<String>,
    pub origination: Option<String>,
    pub created: Option<String>,
}

/// A registration that passed every check.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ValidRegistration {
    pub email: String,
    pub password: String,
    pub key_params: KeyParams,
}

/// The public values the client derives its keys from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeyParams {
    pub identifier: String,
    pub pw_nonce: String,
    pub version: String,
    pub origination: Option<String>,
    pub created: Option<String>,
}

pub(crate) fn validate(registration: Registration) -> Result<ValidRegistration, ServerError> {
    let (Some(email), Some(password)) = (registration.email, registration.password) else {
        return Err(ServerError::MissingCredentials);
    };
    let email = randomtask_crypto::normalise_email(&email).ok_or(ServerError::InvalidEmail)?;
    if registration.version.as_deref() != Some(VERSION) {
        return Err(ServerError::UnsupportedVersion);
    }
    let pw_nonce = registration
        .pw_nonce
        .filter(|nonce| is_seed(nonce))
        .ok_or(ServerError::InvalidKeyParams)?;
    Ok(ValidRegistration {
        key_params: KeyParams {
            identifier: email.clone(),
            pw_nonce,
            version: VERSION.to_owned(),
            origination: registration.origination,
            created: registration.created,
        },
        email,
        password,
    })
}

/// 256 bits as 64 lower-case hex characters, as the client makes it.
fn is_seed(nonce: &str) -> bool {
    nonce.len() == 64 && nonce.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

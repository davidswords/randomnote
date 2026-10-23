//! Signing in, as Standard Notes does: the client asks for the key parameters
//! with a PKCE code challenge, then signs in with the server password and the
//! code verifier, which proves it's the same client that asked.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// How long a code challenge can be used, as in Standard Notes.
pub(crate) const CODE_CHALLENGE_AGE_MS: i64 = 60 * 60 * 1000;

#[derive(Debug, Deserialize)]
pub(crate) struct KeyParamsRequest {
    pub email: Option<String>,
    pub code_challenge: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SignIn {
    pub email: Option<String>,
    pub password: Option<String>,
    pub code_verifier: Option<String>,
}

/// The challenge a verifier answers: base64url, without padding, of the hex
/// SHA-256 of the verifier, as Standard Notes' client computes it.
pub(crate) fn code_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(hex::encode(Sha256::digest(verifier)))
}

/// A seed made up for an email with no account, the same every time, so a
/// stranger can't tell whether the account exists.
pub(crate) fn pseudo_seed(email: &str, secret: &str) -> String {
    hex::encode(Sha256::digest(format!("{email}{secret}")))
}

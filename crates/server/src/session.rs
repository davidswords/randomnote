//! Session tokens, in the Standard Notes format `1:<session uuid>:<token>`.
//! Only SHA-256 hashes of the tokens are stored.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;
/// How long an access token lasts, as in Standard Notes.
pub(crate) const ACCESS_AGE_MS: i64 = 60 * DAY_MS;
/// How long the session can be refreshed, as in Standard Notes.
pub(crate) const REFRESH_AGE_MS: i64 = 365 * DAY_MS;
const TOKEN_BYTES: usize = 48;

/// A new session, holding its tokens in full only until they're sent.
pub(crate) struct NewSession {
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub access_expiration: i64,
    pub refresh_expiration: i64,
}

impl NewSession {
    pub fn start(now: i64) -> Self {
        Self {
            uuid: uuid::Uuid::now_v7().to_string(),
            access_token: random_token(),
            refresh_token: random_token(),
            access_expiration: now + ACCESS_AGE_MS,
            refresh_expiration: now + REFRESH_AGE_MS,
        }
    }

    pub fn full_access_token(&self) -> String {
        format!("1:{}:{}", self.uuid, self.access_token)
    }

    pub fn full_refresh_token(&self) -> String {
        format!("1:{}:{}", self.uuid, self.refresh_token)
    }
}

pub(crate) fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token))
}

fn random_token() -> String {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).expect("the operating system provides randomness");
    URL_SAFE_NO_PAD.encode(bytes)
}

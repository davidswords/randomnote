//! The server's SQLite database: accounts and sessions.

use std::path::Path;

use rusqlite::{Connection, ErrorCode, params};

use crate::error::{ServerError, storage};
use crate::registration::KeyParams;
use crate::session::{NewSession, hash};

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS users (
        uuid TEXT PRIMARY KEY,
        email TEXT NOT NULL UNIQUE,
        encrypted_password TEXT NOT NULL,
        pw_nonce TEXT NOT NULL,
        version TEXT NOT NULL,
        origination TEXT,
        created TEXT,
        created_at INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS sessions (
        uuid TEXT PRIMARY KEY,
        user_uuid TEXT NOT NULL REFERENCES users (uuid),
        access_token_hash TEXT NOT NULL,
        refresh_token_hash TEXT NOT NULL,
        access_expiration INTEGER NOT NULL,
        refresh_expiration INTEGER NOT NULL,
        created_at INTEGER NOT NULL
    );
";

pub struct Database {
    connection: Connection,
}

/// A user about to be stored. The password is already a bcrypt hash.
pub(crate) struct NewUser<'a> {
    pub uuid: &'a str,
    pub encrypted_password: &'a str,
    pub key_params: &'a KeyParams,
}

impl Database {
    /// Opens the database file, creating it and its tables if needed.
    pub fn open(path: &Path) -> Result<Self, ServerError> {
        let connection = Connection::open(path).map_err(storage)?;
        connection.execute_batch(SCHEMA).map_err(storage)?;
        Ok(Self { connection })
    }

    /// Stores the user and their first session together, or neither.
    pub(crate) fn insert_user_with_session(
        &mut self,
        user: &NewUser,
        session: &NewSession,
        now: i64,
    ) -> Result<(), ServerError> {
        let transaction = self.connection.transaction().map_err(storage)?;
        let params = user.key_params;
        transaction
            .execute(
                "INSERT INTO users (uuid, email, encrypted_password, pw_nonce, version,
                    origination, created, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    user.uuid,
                    params.identifier,
                    user.encrypted_password,
                    params.pw_nonce,
                    params.version,
                    params.origination,
                    params.created,
                    now,
                ],
            )
            .map_err(|error| match error.sqlite_error_code() {
                Some(ErrorCode::ConstraintViolation) => ServerError::EmailTaken,
                _ => storage(error),
            })?;
        transaction
            .execute(
                "INSERT INTO sessions (uuid, user_uuid, access_token_hash, refresh_token_hash,
                    access_expiration, refresh_expiration, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    session.uuid,
                    user.uuid,
                    hash(&session.access_token),
                    hash(&session.refresh_token),
                    session.access_expiration,
                    session.refresh_expiration,
                    now,
                ],
            )
            .map_err(storage)?;
        transaction.commit().map_err(storage)
    }
}

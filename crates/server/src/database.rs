//! The server's SQLite database: accounts and sessions.

use std::path::Path;

use rusqlite::{Connection, ErrorCode, OptionalExtension, params};

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
    CREATE TABLE IF NOT EXISTS code_challenges (
        challenge TEXT PRIMARY KEY,
        user_uuid TEXT NOT NULL REFERENCES users (uuid),
        expires_at INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS settings (
        name TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS failed_sign_ins (
        email TEXT PRIMARY KEY,
        failures INTEGER NOT NULL,
        locked_until INTEGER NOT NULL
    );
";

pub struct Database {
    connection: Connection,
}

/// A stored user, found by email.
pub(crate) struct StoredUser {
    pub uuid: String,
    pub encrypted_password: String,
    pub key_params: KeyParams,
}

/// A stored session's token hashes and when the session can last be refreshed.
pub(crate) struct StoredSession {
    pub access_token_hash: String,
    pub refresh_token_hash: String,
    pub refresh_expiration: i64,
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

    /// The secret that made-up key parameters for unknown emails come from.
    /// Made at random the first time it's asked for, then kept, so the
    /// made-up parameters stay the same across restarts.
    pub fn pseudo_key(&self) -> Result<String, ServerError> {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).expect("the operating system provides randomness");
        self.connection
            .execute(
                "INSERT OR IGNORE INTO settings (name, value) VALUES ('pseudo_key', ?1)",
                [hex::encode(bytes)],
            )
            .and_then(|_| {
                self.connection.query_row(
                    "SELECT value FROM settings WHERE name = 'pseudo_key'",
                    [],
                    |row| row.get(0),
                )
            })
            .map_err(storage)
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
        insert_session(&transaction, user.uuid, session, now)?;
        transaction.commit().map_err(storage)
    }

    pub(crate) fn insert_session(
        &self,
        user_uuid: &str,
        session: &NewSession,
        now: i64,
    ) -> Result<(), ServerError> {
        insert_session(&self.connection, user_uuid, session, now)
    }

    pub(crate) fn find_user(&self, email: &str) -> Result<Option<StoredUser>, ServerError> {
        self.connection
            .query_row(
                "SELECT uuid, encrypted_password, pw_nonce, version, origination, created
                 FROM users WHERE email = ?1",
                [email],
                |row| {
                    Ok(StoredUser {
                        uuid: row.get(0)?,
                        encrypted_password: row.get(1)?,
                        key_params: KeyParams {
                            identifier: email.to_owned(),
                            pw_nonce: row.get(2)?,
                            version: row.get(3)?,
                            origination: row.get(4)?,
                            created: row.get(5)?,
                        },
                    })
                },
            )
            .optional()
            .map_err(storage)
    }

    pub(crate) fn store_code_challenge(
        &self,
        challenge: &str,
        user_uuid: &str,
        expires_at: i64,
    ) -> Result<(), ServerError> {
        self.connection
            .execute(
                "INSERT OR REPLACE INTO code_challenges (challenge, user_uuid, expires_at)
                 VALUES (?1, ?2, ?3)",
                params![challenge, user_uuid, expires_at],
            )
            .map(drop)
            .map_err(storage)
    }

    /// Removes the user's code challenge, and says whether it was there and
    /// still in date. Each challenge can be used once.
    pub(crate) fn take_code_challenge(
        &self,
        challenge: &str,
        user_uuid: &str,
        now: i64,
    ) -> Result<bool, ServerError> {
        self.connection
            .query_row(
                "DELETE FROM code_challenges WHERE challenge = ?1 AND user_uuid = ?2
                 RETURNING expires_at > ?3",
                params![challenge, user_uuid, now],
                |row| row.get(0),
            )
            .optional()
            .map(|in_date| in_date == Some(true))
            .map_err(storage)
    }
}

impl Database {
    /// Whether sign-in is locked for the email at `now`.
    pub(crate) fn is_locked(&self, email: &str, now: i64) -> Result<bool, ServerError> {
        self.connection
            .query_row(
                "SELECT locked_until > ?2 FROM failed_sign_ins WHERE email = ?1",
                params![email, now],
                |row| row.get(0),
            )
            .optional()
            .map(|locked| locked == Some(true))
            .map_err(storage)
    }

    /// Counts a failed sign-in, and once there are `limit` in a row, locks the
    /// email until `locked_until` and starts counting again.
    pub(crate) fn record_failed_sign_in(
        &self,
        email: &str,
        limit: i64,
        locked_until: i64,
    ) -> Result<(), ServerError> {
        self.connection
            .execute(
                "INSERT INTO failed_sign_ins (email, failures, locked_until) VALUES (?1, 1, 0)
                 ON CONFLICT (email) DO UPDATE SET failures = failures + 1",
                [email],
            )
            .and_then(|_| {
                self.connection.execute(
                    "UPDATE failed_sign_ins SET failures = 0, locked_until = ?3
                     WHERE email = ?1 AND failures >= ?2",
                    params![email, limit, locked_until],
                )
            })
            .map(drop)
            .map_err(storage)
    }

    pub(crate) fn find_session(&self, uuid: &str) -> Result<Option<StoredSession>, ServerError> {
        self.connection
            .query_row(
                "SELECT access_token_hash, refresh_token_hash, refresh_expiration
                 FROM sessions WHERE uuid = ?1",
                [uuid],
                |row| {
                    Ok(StoredSession {
                        access_token_hash: row.get(0)?,
                        refresh_token_hash: row.get(1)?,
                        refresh_expiration: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(storage)
    }

    /// Replaces the session's tokens and expirations with `session`'s.
    pub(crate) fn renew_session(&self, session: &NewSession) -> Result<(), ServerError> {
        self.connection
            .execute(
                "UPDATE sessions SET access_token_hash = ?2, refresh_token_hash = ?3,
                    access_expiration = ?4, refresh_expiration = ?5
                 WHERE uuid = ?1",
                params![
                    session.uuid,
                    hash(&session.access_token),
                    hash(&session.refresh_token),
                    session.access_expiration,
                    session.refresh_expiration,
                ],
            )
            .map(drop)
            .map_err(storage)
    }

    pub(crate) fn delete_session(&self, uuid: &str) -> Result<(), ServerError> {
        self.connection
            .execute("DELETE FROM sessions WHERE uuid = ?1", [uuid])
            .map(drop)
            .map_err(storage)
    }

    pub(crate) fn clear_failed_sign_ins(&self, email: &str) -> Result<(), ServerError> {
        self.connection
            .execute("DELETE FROM failed_sign_ins WHERE email = ?1", [email])
            .map(drop)
            .map_err(storage)
    }
}

fn insert_session(
    connection: &Connection,
    user_uuid: &str,
    session: &NewSession,
    now: i64,
) -> Result<(), ServerError> {
    connection
        .execute(
            "INSERT INTO sessions (uuid, user_uuid, access_token_hash, refresh_token_hash,
                access_expiration, refresh_expiration, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session.uuid,
                user_uuid,
                hash(&session.access_token),
                hash(&session.refresh_token),
                session.access_expiration,
                session.refresh_expiration,
                now,
            ],
        )
        .map(drop)
        .map_err(storage)
}

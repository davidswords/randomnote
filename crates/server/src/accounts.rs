//! Registering accounts and starting their sessions.

use std::sync::{Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::{Database, NewUser};
use crate::error::{ServerError, storage};
use crate::registration::{KeyParams, Registration, validate};
use crate::session::NewSession;

/// bcrypt's cost for the server password, as in Standard Notes.
const PASSWORD_HASH_COST: u32 = 11;

/// The current time, in milliseconds since the Unix epoch.
pub trait Clock: Send + Sync {
    fn now(&self) -> i64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> i64 {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after 1970");
        i64::try_from(elapsed.as_millis()).expect("the time fits in 64 bits")
    }
}

pub struct Accounts {
    database: Mutex<Database>,
    clock: Box<dyn Clock>,
}

/// What a successful registration or sign-in returns.
pub(crate) struct SignedIn {
    pub user_uuid: String,
    pub key_params: KeyParams,
    pub session: NewSession,
}

impl Accounts {
    pub fn new(database: Database, clock: Box<dyn Clock>) -> Self {
        Self {
            database: Mutex::new(database),
            clock,
        }
    }

    pub(crate) fn register(&self, registration: Registration) -> Result<SignedIn, ServerError> {
        let registration = validate(registration)?;
        let encrypted_password =
            bcrypt::hash(&registration.password, PASSWORD_HASH_COST).map_err(storage)?;
        let user_uuid = uuid::Uuid::now_v7().to_string();
        let now = self.clock.now();
        let session = NewSession::start(now);
        let user = NewUser {
            uuid: &user_uuid,
            encrypted_password: &encrypted_password,
            key_params: &registration.key_params,
        };
        self.database
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert_user_with_session(&user, &session, now)?;
        Ok(SignedIn {
            user_uuid,
            key_params: registration.key_params,
            session,
        })
    }
}

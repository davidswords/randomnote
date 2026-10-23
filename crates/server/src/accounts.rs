//! Registering accounts and starting their sessions.

use std::sync::{Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::{Database, NewUser};
use crate::error::{ServerError, storage};
use crate::registration::{KeyParams, Registration, VERSION, validate};
use crate::session::NewSession;
use crate::sign_in::{
    CODE_CHALLENGE_AGE_MS, KeyParamsRequest, SignIn, code_challenge, pseudo_seed,
};

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
    /// The secret that made-up key parameters for unknown emails come from.
    pseudo_key: String,
}

/// What a successful registration or sign-in returns.
pub(crate) struct SignedIn {
    pub user_uuid: String,
    pub key_params: KeyParams,
    pub session: NewSession,
}

impl Accounts {
    pub fn new(database: Database, clock: Box<dyn Clock>, pseudo_key: String) -> Self {
        Self {
            database: Mutex::new(database),
            clock,
            pseudo_key,
        }
    }

    fn database(&self) -> std::sync::MutexGuard<'_, Database> {
        self.database.lock().unwrap_or_else(PoisonError::into_inner)
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
        self.database()
            .insert_user_with_session(&user, &session, now)?;
        Ok(SignedIn {
            user_uuid,
            key_params: registration.key_params,
            session,
        })
    }

    /// The key parameters to sign in with. Only the public ones are given,
    /// and an email with no account gets made-up ones.
    pub(crate) fn key_params(&self, request: KeyParamsRequest) -> Result<KeyParams, ServerError> {
        let email = request.email.ok_or(ServerError::MissingEmail)?;
        let challenge = request
            .code_challenge
            .ok_or(ServerError::MissingCodeChallenge)?;
        let email = randomtask_crypto::normalise_email(&email).ok_or(ServerError::InvalidEmail)?;
        let database = self.database();
        let Some(user) = database.find_user(&email)? else {
            return Ok(KeyParams {
                pw_nonce: pseudo_seed(&email, &self.pseudo_key),
                identifier: email,
                version: VERSION.to_owned(),
                origination: None,
                created: None,
            });
        };
        let expires_at = self.clock.now() + CODE_CHALLENGE_AGE_MS;
        database.store_code_challenge(&challenge, &user.uuid, expires_at)?;
        Ok(KeyParams {
            origination: None,
            created: None,
            ..user.key_params
        })
    }

    /// Signs in with the server password and the verifier of a code challenge
    /// given with the key parameters, and starts a session.
    pub(crate) fn sign_in(&self, sign_in: SignIn) -> Result<SignedIn, ServerError> {
        let (Some(email), Some(password), Some(verifier)) =
            (sign_in.email, sign_in.password, sign_in.code_verifier)
        else {
            return Err(ServerError::IncompleteSignIn);
        };
        let email =
            randomtask_crypto::normalise_email(&email).ok_or(ServerError::WrongCredentials)?;
        let now = self.clock.now();
        let user = {
            let database = self.database();
            let user = database
                .find_user(&email)?
                .ok_or(ServerError::WrongCredentials)?;
            if !database.take_code_challenge(&code_challenge(&verifier), &user.uuid, now)? {
                return Err(ServerError::WrongCredentials);
            }
            user
        };
        if !bcrypt::verify(&password, &user.encrypted_password).map_err(storage)? {
            return Err(ServerError::WrongCredentials);
        }
        let session = NewSession::start(now);
        self.database().insert_session(&user.uuid, &session, now)?;
        Ok(SignedIn {
            user_uuid: user.uuid,
            key_params: user.key_params,
            session,
        })
    }
}

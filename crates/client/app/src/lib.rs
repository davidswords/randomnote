//! What the randomtask apps do with the account and its tasks: the vault,
//! plus registering the account with the server when there is one. The
//! desktop app and the web app share it, each adding only its own way for
//! the interface to call in.

use std::fmt;
use std::sync::{Mutex, MutexGuard, PoisonError};

use randomtask_api::{ClientError, ServerClient};
use randomtask_store::Task;
use randomtask_vault::{Registration, Session, Vault, VaultError};

/// Why something the user asked for failed. Displays as a message for them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    Vault(VaultError),
    Server(ClientError),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vault(error) => error.fmt(formatter),
            Self::Server(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for AppError {}

impl From<VaultError> for AppError {
    fn from(error: VaultError) -> Self {
        Self::Vault(error)
    }
}

impl From<ClientError> for AppError {
    fn from(error: ClientError) -> Self {
        Self::Server(error)
    }
}

/// The account and its tasks, and the server accounts are registered with,
/// if there is one yet. Without one, accounts stay on the device.
pub struct App {
    vault: Mutex<Vault>,
    server: Option<ServerClient>,
}

impl App {
    pub fn new(vault: Vault, server: Option<ServerClient>) -> Self {
        Self {
            vault: Mutex::new(vault),
            server,
        }
    }

    pub fn has_account(&self) -> Result<bool, AppError> {
        Ok(self.vault().has_account()?)
    }

    /// Creates the account. With a server, it's registered there first, and
    /// nothing is stored on the device unless that succeeds.
    pub async fn create_account(&self, email: &str, password: &str) -> Result<(), AppError> {
        let Some(server) = &self.server else {
            return Ok(self.vault().create_account(email, password)?);
        };
        let account = self.vault().prepare_account(email, password)?;
        let session = server
            .register(&account.key_params(), account.server_password())
            .await?;
        Ok(self
            .vault()
            .create_registered_account(account, &vault_session(session))?)
    }

    /// Unlocks the vault. With a server, an account made on the device before
    /// there was one is then registered. If that fails, unlocking still
    /// succeeds, and the next unlock tries again.
    pub async fn unlock(&self, password: &str) -> Result<(), AppError> {
        let registration = {
            let mut vault = self.vault();
            vault.unlock(password)?;
            vault.pending_registration()?
        };
        if let (Some(server), Some(registration)) = (&self.server, registration)
            && let Ok(session) = register_or_sign_in(server, &registration).await
        {
            self.vault().save_session(&vault_session(session))?;
        }
        Ok(())
    }

    pub fn save_task(&self, task: &Task) -> Result<(), AppError> {
        Ok(self.vault().save_task(task)?)
    }

    pub fn find_task(&self, id: &str) -> Result<Option<Task>, AppError> {
        Ok(self.vault().find_task(id)?)
    }

    pub fn list_tasks(&self) -> Result<Vec<Task>, AppError> {
        Ok(self.vault().list_tasks()?)
    }

    fn vault(&self) -> MutexGuard<'_, Vault> {
        self.vault.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Registers the account, or signs in if it's already registered with the
/// same keys, as when an earlier registration's session was lost.
async fn register_or_sign_in(
    server: &ServerClient,
    registration: &Registration,
) -> Result<randomtask_api::Session, ClientError> {
    let params = registration.key_params();
    match server
        .register(&params, registration.server_password())
        .await
    {
        Err(ClientError::Refused(_)) => {
            let challenge = server.key_params(&params.identifier).await?;
            server
                .sign_in(challenge, registration.server_password())
                .await
        }
        result => result,
    }
}

fn vault_session(session: randomtask_api::Session) -> Session {
    Session {
        access_token: session.access_token,
        refresh_token: session.refresh_token,
        access_expiration: session.access_expiration,
        refresh_expiration: session.refresh_expiration,
    }
}

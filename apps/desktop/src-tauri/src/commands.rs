//! Tauri commands that give the web view access to the account and its tasks.

use std::sync::{Mutex, MutexGuard};

use randomtask_api::{ClientError, ServerClient, SignInChallenge};
use randomtask_store::Task;
use randomtask_vault::{Registration, Session, Vault};
use tauri::State;

/// The vault, shared by all commands.
pub struct VaultState(Mutex<Vault>);

impl VaultState {
    pub fn new(vault: Vault) -> Self {
        Self(Mutex::new(vault))
    }

    fn vault(&self) -> Result<MutexGuard<'_, Vault>, String> {
        self.0.lock().map_err(to_message)
    }
}

/// The randomtask server accounts are registered with, if there is one yet.
/// Without one, accounts stay on the device.
pub struct Server(Option<ServerClient>);

impl Server {
    pub fn none() -> Self {
        Self(None)
    }

    pub fn at(url: &str) -> Self {
        Self(Some(ServerClient::new(url)))
    }
}

#[tauri::command]
pub fn has_account(state: State<'_, VaultState>) -> Result<bool, String> {
    state.vault()?.has_account().map_err(to_message)
}

/// Creates the account. With a server, it's registered there first, and
/// nothing is stored on the device unless that succeeds.
#[tauri::command]
pub async fn create_account(
    state: State<'_, VaultState>,
    server: State<'_, Server>,
    email: String,
    password: String,
) -> Result<(), String> {
    let Some(client) = &server.0 else {
        return state
            .vault()?
            .create_account(&email, &password)
            .map_err(to_message);
    };
    let account = state
        .vault()?
        .prepare_account(&email, &password)
        .map_err(to_message)?;
    let session = client
        .register(&account.key_params(), account.server_password())
        .await
        .map_err(to_message)?;
    state
        .vault()?
        .create_registered_account(account, &vault_session(session))
        .map_err(to_message)
}

/// Unlocks the vault. With a server, an account made on the device before
/// there was one is then registered. If that fails, unlocking still succeeds,
/// and the next unlock tries again.
#[tauri::command]
pub async fn unlock(
    state: State<'_, VaultState>,
    server: State<'_, Server>,
    password: String,
) -> Result<(), String> {
    let registration = {
        let mut vault = state.vault()?;
        vault.unlock(&password).map_err(to_message)?;
        vault.pending_registration().map_err(to_message)?
    };
    if let (Some(client), Some(registration)) = (&server.0, registration)
        && let Ok(session) = register_or_sign_in(client, &registration).await
    {
        state
            .vault()?
            .save_session(&vault_session(session))
            .map_err(to_message)?;
    }
    Ok(())
}

/// Registers the account, or signs in if it's already registered with the
/// same keys, as when an earlier registration's session was lost.
async fn register_or_sign_in(
    client: &ServerClient,
    registration: &Registration,
) -> Result<randomtask_api::Session, ClientError> {
    let params = registration.key_params();
    match client
        .register(&params, registration.server_password())
        .await
    {
        Err(ClientError::Refused(_)) => {
            let challenge: SignInChallenge = client.key_params(&params.identifier).await?;
            client
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

#[tauri::command]
pub fn save_task(state: State<'_, VaultState>, task: Task) -> Result<(), String> {
    state.vault()?.save_task(&task).map_err(to_message)
}

#[tauri::command]
pub fn find_task(state: State<'_, VaultState>, id: String) -> Result<Option<Task>, String> {
    state.vault()?.find_task(&id).map_err(to_message)
}

#[tauri::command]
pub fn list_tasks(state: State<'_, VaultState>) -> Result<Vec<Task>, String> {
    state.vault()?.list_tasks().map_err(to_message)
}

fn to_message(error: impl ToString) -> String {
    error.to_string()
}

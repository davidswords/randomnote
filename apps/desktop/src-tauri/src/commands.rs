//! Tauri commands that give the web view access to the account and its tasks.

use std::sync::{Mutex, MutexGuard};

use randomtask_store::Task;
use randomtask_vault::Vault;
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

#[tauri::command]
pub fn has_account(state: State<'_, VaultState>) -> Result<bool, String> {
    state.vault()?.has_account().map_err(to_message)
}

#[tauri::command]
pub fn create_account(
    state: State<'_, VaultState>,
    email: String,
    password: String,
) -> Result<(), String> {
    state
        .vault()?
        .create_account(&email, &password)
        .map_err(to_message)
}

#[tauri::command]
pub fn unlock(state: State<'_, VaultState>, password: String) -> Result<(), String> {
    state.vault()?.unlock(&password).map_err(to_message)
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

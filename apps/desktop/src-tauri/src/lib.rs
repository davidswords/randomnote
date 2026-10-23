mod app;
mod commands;

use std::error::Error;
use std::fs;
use std::path::Path;

pub use app::run;
pub use commands::VaultState;
use randomtask_store::EncryptedStore;
use randomtask_vault::Vault;
use tauri::{Builder, Runtime};

/// Registers the commands the web view uses to reach the account and its tasks.
pub fn with_commands<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        commands::has_account,
        commands::create_account,
        commands::unlock,
        commands::save_task,
        commands::find_task,
        commands::list_tasks
    ])
}

/// Opens the vault in `data_directory`, creating the directory if needed.
/// The vault starts locked.
pub fn open_vault(data_directory: &Path) -> Result<Vault, Box<dyn Error>> {
    fs::create_dir_all(data_directory)?;
    let store = EncryptedStore::open(&data_directory.join("randomtask.db"))?;
    Ok(Vault::new(store))
}

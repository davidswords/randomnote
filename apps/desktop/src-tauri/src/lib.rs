mod app;
mod commands;

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

pub use app::run;
pub use commands::{Server, VaultState};
use randomtask_store::EncryptedStore;
use randomtask_vault::Vault;
use tauri::{Builder, Runtime};

/// The bundle ID the app had as randomnote, up to 0.2.0, which named its data directory.
const RANDOMNOTE_BUNDLE_ID: &str = "engineering.swords.randomnote";

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

/// Deletes the tasks that versions before 0.3.0 kept in plain text: randomnote
/// 0.2.0's data directory, beside `data_directory` and named after its bundle
/// ID, and randomtask 0.2.1's `tasks.db` inside `data_directory`. Does nothing
/// for whichever isn't there.
pub fn remove_unencrypted_data(data_directory: &Path) -> io::Result<()> {
    // Each is attempted even if the other fails.
    let randomnote = ignore_missing(fs::remove_dir_all(
        data_directory.with_file_name(RANDOMNOTE_BUNDLE_ID),
    ));
    let tasks_file = ignore_missing(fs::remove_file(data_directory.join("tasks.db")));
    randomnote.and(tasks_file)
}

fn ignore_missing(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

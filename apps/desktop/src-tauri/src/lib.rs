mod commands;

use std::error::Error;
use std::fs;
use std::path::Path;

pub use commands::TaskState;
use randomnote_store::TaskStore;
use tauri::{Builder, Manager, Runtime};

/// Registers the commands the web view uses to reach the task store.
pub fn with_task_commands<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        commands::save_task,
        commands::find_task,
        commands::list_tasks
    ])
}

/// Opens the task database in `data_directory`, creating the directory if needed.
pub fn open_task_database(data_directory: &Path) -> Result<TaskStore, Box<dyn Error>> {
    fs::create_dir_all(data_directory)?;
    Ok(TaskStore::open(&data_directory.join("tasks.db"))?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    with_task_commands(tauri::Builder::default())
        .setup(|app| {
            let store = open_task_database(&app.path().app_data_dir()?)?;
            app.manage(TaskState::new(store));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

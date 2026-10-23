mod app;
mod commands;

use std::error::Error;
use std::fs;
use std::path::Path;

pub use app::run;
pub use commands::TaskState;
use randomtask_store::TaskStore;
use tauri::{Builder, Runtime};

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

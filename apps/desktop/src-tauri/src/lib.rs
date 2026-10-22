mod commands;

pub use commands::TaskState;
use tauri::{Builder, Runtime};

/// Registers the commands the web view uses to reach the task store.
pub fn with_task_commands<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        commands::save_task,
        commands::find_task,
        commands::list_tasks
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

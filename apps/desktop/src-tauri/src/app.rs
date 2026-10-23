//! Starting the app. Only the end-to-end tests reach this, so it is kept apart
//! from the code covered by unit and integration tests.

use tauri::Manager;

use crate::{TaskState, open_task_database, with_task_commands};

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

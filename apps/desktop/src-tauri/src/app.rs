//! Starting the app. Only the end-to-end tests reach this, so it is kept apart
//! from the code covered by unit and integration tests.

use tauri::Manager;

use crate::{VaultState, open_vault, with_commands};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    with_commands(tauri::Builder::default())
        .setup(|app| {
            let vault = open_vault(&app.path().app_data_dir()?)?;
            app.manage(VaultState::new(vault));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

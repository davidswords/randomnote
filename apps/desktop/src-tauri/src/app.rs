//! Starting the app. Only the end-to-end tests reach this, so it is kept apart
//! from the code covered by unit and integration tests.

use tauri::Manager;

use randomtask_api::ServerClient;
use randomtask_app::App;

use crate::{open_vault, remove_unencrypted_data, with_commands};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    with_commands(tauri::Builder::default())
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            // If this fails the app still opens, and tries again next launch.
            let _ = remove_unencrypted_data(&data_directory);
            let vault = open_vault(&data_directory)?;
            // Accounts stay on the device until a server is set, as releases
            // do until api.randomtask.ch is live.
            let server = std::env::var("RANDOMTASK_SERVER")
                .ok()
                .map(|url| ServerClient::new(&url));
            app.manage(App::new(vault, server));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

//! Tauri commands that give the web view access to the account and its tasks,
//! through the app shared with the web version.

use randomtask_app::App;
use randomtask_store::Task;
use tauri::State;

#[tauri::command]
pub fn has_account(app: State<'_, App>) -> Result<bool, String> {
    app.has_account().map_err(to_message)
}

/// Creates the account. With a server, it's registered there first.
#[tauri::command]
pub async fn create_account(
    app: State<'_, App>,
    email: String,
    password: String,
) -> Result<(), String> {
    app.create_account(&email, &password)
        .await
        .map_err(to_message)
}

/// Unlocks the vault. With a server, a device-only account is then registered.
#[tauri::command]
pub async fn unlock(app: State<'_, App>, password: String) -> Result<(), String> {
    app.unlock(&password).await.map_err(to_message)
}

#[tauri::command]
pub fn save_task(app: State<'_, App>, task: Task) -> Result<(), String> {
    app.save_task(&task).map_err(to_message)
}

#[tauri::command]
pub fn find_task(app: State<'_, App>, id: String) -> Result<Option<Task>, String> {
    app.find_task(&id).map_err(to_message)
}

#[tauri::command]
pub fn list_tasks(app: State<'_, App>) -> Result<Vec<Task>, String> {
    app.list_tasks().map_err(to_message)
}

fn to_message(error: impl ToString) -> String {
    error.to_string()
}

//! Tauri commands that give the web view access to the task store.

use std::sync::{Mutex, MutexGuard};

use randomnote_store::{Task, TaskStore};
use tauri::State;

/// The task store, shared by all commands.
pub struct TaskState(Mutex<TaskStore>);

impl TaskState {
    pub fn new(store: TaskStore) -> Self {
        Self(Mutex::new(store))
    }

    fn store(&self) -> Result<MutexGuard<'_, TaskStore>, String> {
        self.0.lock().map_err(to_message)
    }
}

#[tauri::command]
pub fn save_task(state: State<'_, TaskState>, task: Task) -> Result<(), String> {
    state.store()?.save(&task).map_err(to_message)
}

#[tauri::command]
pub fn find_task(state: State<'_, TaskState>, id: String) -> Result<Option<Task>, String> {
    state.store()?.find_by_id(&id).map_err(to_message)
}

#[tauri::command]
pub fn list_tasks(state: State<'_, TaskState>) -> Result<Vec<Task>, String> {
    state.store()?.list().map_err(to_message)
}

fn to_message(error: impl ToString) -> String {
    error.to_string()
}

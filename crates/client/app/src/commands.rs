//! The app's commands by name, with JSON arguments and results. The names,
//! argument keys and JSON match the desktop app's Tauri commands, so the web
//! app's worker can pass messages straight through and both apps share the
//! same interface adapters.

use randomtask_store::Task;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::App;

#[derive(Deserialize)]
struct NewAccount {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct Password {
    password: String,
}

#[derive(Deserialize)]
struct TaskArgument {
    task: Task,
}

#[derive(Deserialize)]
struct Id {
    id: String,
}

impl App {
    /// Runs `command` with `args`, giving its result as JSON, or the message
    /// to show for why it failed.
    pub async fn invoke(&self, command: &str, args: Value) -> Result<Value, String> {
        let result = match command {
            "has_account" => self.has_account().map(Value::Bool),
            "create_account" => {
                let NewAccount { email, password } = arguments(command, args)?;
                self.create_account(&email, &password)
                    .await
                    .map(|()| Value::Null)
            }
            "unlock" => {
                let Password { password } = arguments(command, args)?;
                self.unlock(&password).await.map(|()| Value::Null)
            }
            "save_task" => {
                let TaskArgument { task } = arguments(command, args)?;
                self.save_task(&task).map(|()| Value::Null)
            }
            "find_task" => {
                let Id { id } = arguments(command, args)?;
                self.find_task(&id).map(|task| to_json(&task))
            }
            "list_tasks" => self.list_tasks().map(|tasks| to_json(&tasks)),
            _ => return Err(format!("Unknown command: {command}")),
        };
        result.map_err(|error| error.to_string())
    }
}

fn arguments<T: DeserializeOwned>(command: &str, args: Value) -> Result<T, String> {
    serde_json::from_value(args)
        .map_err(|error| format!("Invalid arguments for {command}: {error}"))
}

fn to_json(value: &impl serde::Serialize) -> Value {
    serde_json::to_value(value).expect("tasks serialise to JSON")
}

//! The randomtask app compiled to WebAssembly, for the web app. It runs in a
//! dedicated Web Worker, keeps its SQLite database in the browser's private
//! file storage (OPFS), and takes the same commands as the desktop app.
//!
//! Everything here is glue that only a browser can run, so it's tested by
//! the web app's end-to-end tests. What it calls is tested natively in
//! `randomtask-app`.

#![cfg(all(target_family = "wasm", target_os = "unknown"))]

use std::path::Path;
use std::rc::Rc;

use randomtask_api::ServerClient;
use randomtask_app::App;
use randomtask_store::EncryptedStore;
use randomtask_vault::Vault;
use sqlite_wasm_vfs::sahpool::{OpfsSAHPoolCfgBuilder, install};
use wasm_bindgen::prelude::*;

/// The OPFS directory that holds the database.
const DIRECTORY: &str = "randomtask";
const DATABASE: &str = "randomtask.db";

/// The app, opened on the browser's storage.
#[wasm_bindgen]
pub struct WebApp(Rc<App>);

#[wasm_bindgen]
impl WebApp {
    /// Opens the app's database in OPFS, registering accounts with `server`
    /// if one is given. Must run in a dedicated worker.
    pub async fn open(server: Option<String>) -> Result<WebApp, String> {
        let options = OpfsSAHPoolCfgBuilder::new().directory(DIRECTORY).build();
        install::<sqlite_wasm_rs::WasmOsCallback>(&options, true)
            .await
            .map_err(|error| format!("Couldn't open the browser's storage: {error}"))?;
        let store = EncryptedStore::open(Path::new(DATABASE)).map_err(|error| error.to_string())?;
        let server = server.map(|url| ServerClient::new(&url));
        Ok(WebApp(Rc::new(App::new(Vault::new(store), server))))
    }

    /// Runs `command` with `args` given as JSON, resolving to its result as
    /// JSON or rejecting with the message to show the user.
    pub fn invoke(&self, command: String, args: String) -> js_sys::Promise {
        let app = Rc::clone(&self.0);
        wasm_bindgen_futures::future_to_promise(async move {
            let args = serde_json::from_str(&args).map_err(|error| error.to_string())?;
            let result = app.invoke(&command, args).await?;
            Ok(JsValue::from_str(&result.to_string()))
        })
    }
}

//! Runs the server. Configured with environment variables:
//! `RANDOMTASK_DATABASE` (the SQLite file), `RANDOMTASK_ADDRESS`, and
//! optionally `RANDOMTASK_PSEUDO_KEY`, the secret that made-up key parameters
//! for unknown emails come from. Without it, the database keeps its own.
//! `RANDOMTASK_ALLOWED_ORIGINS` lists, separated by commas, the origins of web
//! pages that may call the server from a browser, such as the web app's.

use std::path::PathBuf;

use randomtask_server::{Accounts, Database, SystemClock, cors, router};

#[tokio::main]
async fn main() {
    let database = std::env::var("RANDOMTASK_DATABASE").unwrap_or("randomtask-server.db".into());
    let address = std::env::var("RANDOMTASK_ADDRESS").unwrap_or("127.0.0.1:8080".into());
    let database = Database::open(&PathBuf::from(database)).expect("the database opens");
    let pseudo_key = match std::env::var("RANDOMTASK_PSEUDO_KEY") {
        Ok(key) => key,
        Err(_) => database
            .pseudo_key()
            .expect("the database has a pseudo key"),
    };
    let accounts = Accounts::new(database, Box::new(SystemClock), pseudo_key);
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("the address is free");
    println!("randomtask server listening on {address}");
    let origins = std::env::var("RANDOMTASK_ALLOWED_ORIGINS").unwrap_or_default();
    let origins: Vec<&str> = origins
        .split(',')
        .filter(|origin| !origin.is_empty())
        .collect();
    let app = router(accounts).layer(cors(&origins).expect("the allowed origins are valid"));
    axum::serve(listener, app).await.expect("the server runs");
}

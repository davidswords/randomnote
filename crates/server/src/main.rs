//! Runs the server. Configured with environment variables:
//! `RANDOMTASK_DATABASE` (the SQLite file), `RANDOMTASK_ADDRESS`, and
//! `RANDOMTASK_PSEUDO_KEY`, a long random secret that made-up key parameters
//! for unknown emails come from. It must stay the same across restarts.

use std::path::PathBuf;

use randomtask_server::{Accounts, Database, SystemClock, router};

#[tokio::main]
async fn main() {
    let database = std::env::var("RANDOMTASK_DATABASE").unwrap_or("randomtask-server.db".into());
    let address = std::env::var("RANDOMTASK_ADDRESS").unwrap_or("127.0.0.1:8080".into());
    let database = Database::open(&PathBuf::from(database)).expect("the database opens");
    let pseudo_key = std::env::var("RANDOMTASK_PSEUDO_KEY").expect("RANDOMTASK_PSEUDO_KEY is set");
    let accounts = Accounts::new(database, Box::new(SystemClock), pseudo_key);
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("the address is free");
    println!("randomtask server listening on {address}");
    axum::serve(listener, router(accounts))
        .await
        .expect("the server runs");
}

//! Opening the server's database file.

use randomtask_server::{Database, ServerError};

#[test]
fn reports_a_database_that_cannot_be_opened() {
    let directory = tempfile::tempdir().unwrap();

    let result = Database::open(&directory.path().join("missing").join("server.db"));

    assert!(matches!(result, Err(ServerError::Storage(_))));
}

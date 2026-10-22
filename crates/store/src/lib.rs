//! SQLite storage for randomnote.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Row, params};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub title: String,
}

pub struct TaskStore {
    connection: Connection,
}

impl TaskStore {
    /// Opens the database file at `path`, creating it if needed.
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        Self::with_schema(Connection::open(path)?)
    }

    /// Opens a database that lives only in memory, for tests.
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        Self::with_schema(Connection::open_in_memory()?)
    }

    fn with_schema(connection: Connection) -> rusqlite::Result<Self> {
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL
            ) STRICT;",
        )?;
        Ok(Self { connection })
    }

    pub fn save(&self, task: &Task) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO tasks (id, title) VALUES (?1, ?2)
             ON CONFLICT (id) DO UPDATE SET title = excluded.title",
            params![task.id, task.title],
        )?;
        Ok(())
    }

    pub fn find_by_id(&self, id: &str) -> rusqlite::Result<Option<Task>> {
        self.connection
            .query_row(
                "SELECT id, title FROM tasks WHERE id = ?1",
                params![id],
                task_from_row,
            )
            .optional()
    }

    pub fn list(&self) -> rusqlite::Result<Vec<Task>> {
        self.connection
            .prepare("SELECT id, title FROM tasks")?
            .query_map([], task_from_row)?
            .collect()
    }
}

fn task_from_row(row: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
    })
}

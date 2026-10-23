//! Storage for the app's data. Once an account exists, everything here is
//! encrypted apart from the account's key parameters, which aren't secret.
//! Before that, tasks are kept unencrypted, as nothing can encrypt them yet.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::Task;

/// The key parameters needed to derive the account's keys from its password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub identifier: String,
    pub seed: String,
    pub version: String,
}

/// An items key, encrypted under the master key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredItemsKey {
    pub id: String,
    pub enc_item_key: String,
    pub content: String,
}

/// An item, encrypted under its own key, which an items key encrypts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredItem {
    pub id: String,
    pub items_key_id: String,
    pub enc_item_key: String,
    pub content: String,
}

pub struct EncryptedStore {
    connection: Connection,
}

impl EncryptedStore {
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
            "CREATE TABLE IF NOT EXISTS account (
                singleton INTEGER PRIMARY KEY,
                identifier TEXT NOT NULL,
                seed TEXT NOT NULL,
                version TEXT NOT NULL
            ) STRICT;
            CREATE TABLE IF NOT EXISTS items_keys (
                id TEXT PRIMARY KEY NOT NULL,
                enc_item_key TEXT NOT NULL,
                content TEXT NOT NULL
            ) STRICT;
            CREATE TABLE IF NOT EXISTS items (
                id TEXT PRIMARY KEY NOT NULL,
                items_key_id TEXT NOT NULL,
                enc_item_key TEXT NOT NULL,
                content TEXT NOT NULL
            ) STRICT;
            CREATE TABLE IF NOT EXISTS unencrypted_tasks (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL
            ) STRICT;",
        )?;
        Ok(Self { connection })
    }

    pub fn account(&self) -> rusqlite::Result<Option<Account>> {
        self.connection
            .query_row("SELECT identifier, seed, version FROM account", [], |row| {
                Ok(Account {
                    identifier: row.get(0)?,
                    seed: row.get(1)?,
                    version: row.get(2)?,
                })
            })
            .optional()
    }

    /// Saves the device's only account. Fails if one already exists.
    pub fn create_account(&self, account: &Account) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO account (singleton, identifier, seed, version) VALUES (1, ?1, ?2, ?3)",
            params![account.identifier, account.seed, account.version],
        )?;
        Ok(())
    }

    pub fn save_items_key(&self, key: &StoredItemsKey) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO items_keys (id, enc_item_key, content) VALUES (?1, ?2, ?3)",
            params![key.id, key.enc_item_key, key.content],
        )?;
        Ok(())
    }

    pub fn items_keys(&self) -> rusqlite::Result<Vec<StoredItemsKey>> {
        self.connection
            .prepare("SELECT id, enc_item_key, content FROM items_keys")?
            .query_map([], items_key_from_row)?
            .collect()
    }

    /// Saves an item, replacing any item with the same ID.
    pub fn save_item(&self, item: &StoredItem) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO items (id, items_key_id, enc_item_key, content) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (id) DO UPDATE SET
                items_key_id = excluded.items_key_id,
                enc_item_key = excluded.enc_item_key,
                content = excluded.content",
            params![item.id, item.items_key_id, item.enc_item_key, item.content],
        )?;
        Ok(())
    }

    pub fn find_item(&self, id: &str) -> rusqlite::Result<Option<StoredItem>> {
        self.connection
            .query_row(
                "SELECT id, items_key_id, enc_item_key, content FROM items WHERE id = ?1",
                params![id],
                item_from_row,
            )
            .optional()
    }

    pub fn items(&self) -> rusqlite::Result<Vec<StoredItem>> {
        self.connection
            .prepare("SELECT id, items_key_id, enc_item_key, content FROM items")?
            .query_map([], item_from_row)?
            .collect()
    }

    /// Saves a task kept before the account exists, replacing any with the same ID.
    pub fn save_unencrypted_task(&self, task: &Task) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO unencrypted_tasks (id, title) VALUES (?1, ?2)
             ON CONFLICT (id) DO UPDATE SET title = excluded.title",
            params![task.id, task.title],
        )?;
        Ok(())
    }

    pub fn find_unencrypted_task(&self, id: &str) -> rusqlite::Result<Option<Task>> {
        self.connection
            .query_row(
                "SELECT id, title FROM unencrypted_tasks WHERE id = ?1",
                params![id],
                task_from_row,
            )
            .optional()
    }

    pub fn unencrypted_tasks(&self) -> rusqlite::Result<Vec<Task>> {
        self.connection
            .prepare("SELECT id, title FROM unencrypted_tasks")?
            .query_map([], task_from_row)?
            .collect()
    }

    /// Deletes every unencrypted task, then rebuilds the file so their
    /// contents don't linger in its free pages.
    pub fn wipe_unencrypted_tasks(&self) -> rusqlite::Result<()> {
        self.connection
            .execute_batch("DELETE FROM unencrypted_tasks; VACUUM;")
    }
}

fn task_from_row(row: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
    })
}

fn items_key_from_row(row: &Row) -> rusqlite::Result<StoredItemsKey> {
    Ok(StoredItemsKey {
        id: row.get(0)?,
        enc_item_key: row.get(1)?,
        content: row.get(2)?,
    })
}

fn item_from_row(row: &Row) -> rusqlite::Result<StoredItem> {
    Ok(StoredItem {
        id: row.get(0)?,
        items_key_id: row.get(1)?,
        enc_item_key: row.get(2)?,
        content: row.get(3)?,
    })
}

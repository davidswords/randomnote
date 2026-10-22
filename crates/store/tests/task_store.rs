//! TaskStore against a real SQLite database. Mirrors the TaskRepository
//! contract in packages/core.

use randomnote_store::{Task, TaskStore};

fn a_task(id: &str, title: &str) -> Task {
    Task {
        id: id.to_owned(),
        title: title.to_owned(),
    }
}

#[test]
fn finds_a_saved_task_by_its_id() {
    let store = TaskStore::open_in_memory().unwrap();
    let task = a_task("task-1", "Buy milk");

    store.save(&task).unwrap();

    assert_eq!(store.find_by_id("task-1").unwrap(), Some(task));
}

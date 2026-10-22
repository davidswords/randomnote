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

#[test]
fn finds_the_task_with_the_given_id_among_several() {
    let store = TaskStore::open_in_memory().unwrap();
    store.save(&a_task("task-1", "Buy milk")).unwrap();
    store.save(&a_task("task-2", "Call the dentist")).unwrap();

    assert_eq!(
        store.find_by_id("task-2").unwrap(),
        Some(a_task("task-2", "Call the dentist"))
    );
}

#[test]
fn finds_nothing_for_an_id_that_was_never_saved() {
    let store = TaskStore::open_in_memory().unwrap();

    assert_eq!(store.find_by_id("missing").unwrap(), None);
}

#[test]
fn lists_nothing_when_no_task_was_saved() {
    let store = TaskStore::open_in_memory().unwrap();

    assert_eq!(store.list().unwrap(), vec![]);
}

#[test]
fn lists_every_saved_task() {
    let store = TaskStore::open_in_memory().unwrap();
    let milk = a_task("task-1", "Buy milk");
    let dentist = a_task("task-2", "Call the dentist");

    store.save(&milk).unwrap();
    store.save(&dentist).unwrap();

    let tasks = store.list().unwrap();
    assert_eq!(tasks.len(), 2);
    assert!(tasks.contains(&milk));
    assert!(tasks.contains(&dentist));
}

#[test]
fn updates_a_task_in_place_when_it_is_saved_again_with_the_same_id() {
    let store = TaskStore::open_in_memory().unwrap();
    store.save(&a_task("task-1", "Buy milk")).unwrap();

    store.save(&a_task("task-1", "Buy oat milk")).unwrap();

    assert_eq!(
        store.find_by_id("task-1").unwrap(),
        Some(a_task("task-1", "Buy oat milk"))
    );
    assert_eq!(store.list().unwrap(), vec![a_task("task-1", "Buy oat milk")]);
}

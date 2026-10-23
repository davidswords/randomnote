//! Opening the task database in the app's data directory, on a real disk.

use randomtask_lib::open_task_database;
use randomtask_store::Task;

#[test]
fn creates_the_data_directory_and_keeps_tasks_across_restarts() {
    let parent = tempfile::tempdir().unwrap();
    let data_directory = parent.path().join("engineering.swords.randomtask");
    let task = Task {
        id: "task-1".to_owned(),
        title: "Buy milk".to_owned(),
    };

    open_task_database(&data_directory)
        .unwrap()
        .save(&task)
        .unwrap();
    let reopened = open_task_database(&data_directory).unwrap();

    assert_eq!(reopened.list().unwrap(), vec![task]);
    assert!(data_directory.join("tasks.db").exists());
}

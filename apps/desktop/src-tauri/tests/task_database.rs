//! Opening the vault in the app's data directory, on a real disk.

use randomtask_lib::open_vault;
use randomtask_store::Task;

#[test]
fn creates_the_data_directory_and_keeps_tasks_across_restarts() {
    let parent = tempfile::tempdir().unwrap();
    let data_directory = parent.path().join("engineering.swords.randomtask");
    let task = Task {
        id: "task-1".to_owned(),
        title: "Buy milk".to_owned(),
    };
    {
        let mut vault = open_vault(&data_directory).unwrap();
        vault
            .create_account("david", "correct horse battery staple")
            .unwrap();
        vault.save_task(&task).unwrap();
    }

    let mut reopened = open_vault(&data_directory).unwrap();
    reopened.unlock("correct horse battery staple").unwrap();

    assert_eq!(reopened.list_tasks(), Ok(vec![task]));
    assert!(data_directory.join("randomtask.db").exists());
}

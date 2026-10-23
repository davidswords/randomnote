//! Removing the tasks that versions before 0.3.0 stored in plain text, on a
//! real disk. randomnote 0.2.0 kept them in a data directory beside the app's
//! own, named after its bundle ID; randomtask 0.2.1 kept them in `tasks.db`
//! inside the app's own data directory.

use std::fs;
use std::path::PathBuf;

use randomtask_lib::remove_unencrypted_data;

struct DataDirectories {
    _parent: tempfile::TempDir,
    randomtask: PathBuf,
    randomnote: PathBuf,
}

fn data_directories() -> DataDirectories {
    let parent = tempfile::tempdir().unwrap();
    let randomtask = parent.path().join("engineering.swords.randomtask");
    let randomnote = parent.path().join("engineering.swords.randomnote");
    DataDirectories {
        _parent: parent,
        randomtask,
        randomnote,
    }
}

#[test]
fn deletes_the_randomnote_data_directory() {
    let directories = data_directories();
    fs::create_dir_all(&directories.randomnote).unwrap();
    fs::write(directories.randomnote.join("tasks.db"), "Buy milk").unwrap();

    remove_unencrypted_data(&directories.randomtask).unwrap();

    assert!(!directories.randomnote.exists());
}

#[test]
fn leaves_the_app_data_directory_alone() {
    let directories = data_directories();
    fs::create_dir_all(&directories.randomnote).unwrap();
    fs::create_dir_all(&directories.randomtask).unwrap();
    fs::write(directories.randomtask.join("randomtask.db"), "ciphertext").unwrap();

    remove_unencrypted_data(&directories.randomtask).unwrap();

    assert!(directories.randomtask.join("randomtask.db").exists());
}

#[test]
fn does_nothing_when_there_is_no_randomnote_data() {
    let directories = data_directories();

    assert!(remove_unencrypted_data(&directories.randomtask).is_ok());
}

#[test]
fn reports_an_error_when_the_data_cannot_be_deleted() {
    let directories = data_directories();
    fs::create_dir_all(directories.randomnote.parent().unwrap()).unwrap();
    fs::write(&directories.randomnote, "not a directory").unwrap();

    assert!(remove_unencrypted_data(&directories.randomtask).is_err());
}

#[test]
fn deletes_the_unencrypted_tasks_file_from_randomtask_0_2_1() {
    let directories = data_directories();
    fs::create_dir_all(&directories.randomtask).unwrap();
    fs::write(directories.randomtask.join("tasks.db"), "Buy milk").unwrap();

    remove_unencrypted_data(&directories.randomtask).unwrap();

    assert!(!directories.randomtask.join("tasks.db").exists());
}

#[test]
fn reports_an_error_when_the_tasks_file_cannot_be_deleted() {
    let directories = data_directories();
    fs::create_dir_all(directories.randomtask.join("tasks.db")).unwrap();

    assert!(remove_unencrypted_data(&directories.randomtask).is_err());
}

#[test]
fn still_deletes_the_tasks_file_when_the_randomnote_data_cannot_be_deleted() {
    let directories = data_directories();
    fs::create_dir_all(&directories.randomtask).unwrap();
    fs::write(&directories.randomnote, "not a directory").unwrap();
    fs::write(directories.randomtask.join("tasks.db"), "Buy milk").unwrap();

    assert!(remove_unencrypted_data(&directories.randomtask).is_err());
    assert!(!directories.randomtask.join("tasks.db").exists());
}

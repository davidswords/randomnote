//! Removing the data that randomnote 0.2.0 left in plain text, on a real disk.
//! The app's data directory and randomnote's sit side by side, named after
//! each app's bundle ID.

use std::fs;
use std::path::PathBuf;

use randomtask_lib::remove_randomnote_data;

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

    remove_randomnote_data(&directories.randomtask).unwrap();

    assert!(!directories.randomnote.exists());
}

#[test]
fn leaves_the_app_data_directory_alone() {
    let directories = data_directories();
    fs::create_dir_all(&directories.randomnote).unwrap();
    fs::create_dir_all(&directories.randomtask).unwrap();
    fs::write(directories.randomtask.join("randomtask.db"), "ciphertext").unwrap();

    remove_randomnote_data(&directories.randomtask).unwrap();

    assert!(directories.randomtask.join("randomtask.db").exists());
}

#[test]
fn does_nothing_when_there_is_no_randomnote_data() {
    let directories = data_directories();

    assert!(remove_randomnote_data(&directories.randomtask).is_ok());
}

#[test]
fn reports_an_error_when_the_data_cannot_be_deleted() {
    let directories = data_directories();
    fs::create_dir_all(directories.randomnote.parent().unwrap()).unwrap();
    fs::write(&directories.randomnote, "not a directory").unwrap();

    assert!(remove_randomnote_data(&directories.randomtask).is_err());
}

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::File;
use tempfile::tempdir;

#[test]
fn test_cli_basic_rename() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_1.txt");
    let file2 = dir.path().join("test_2.txt");
    File::create(&file1).unwrap();
    File::create(&file2).unwrap();

    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("test_(\\d+).txt")
        .arg("-p")
        .arg("file_${1}.txt")
        .arg("--history-file")
        .arg(dir.path().join("history.json"));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_1.txt").and(predicate::str::contains("file_1.txt")))
        .stdout(predicate::str::contains("test_2.txt").and(predicate::str::contains("file_2.txt")));

    assert!(dir.path().join("file_1.txt").exists());
    assert!(dir.path().join("file_2.txt").exists());
    assert!(!file1.exists());
    assert!(!file2.exists());
}

#[test]
fn test_cli_dry_run() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_1.txt");
    File::create(&file1).unwrap();

    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("test_(\\d+).txt")
        .arg("-p")
        .arg("file_${1}.txt")
        .arg("-d")
        .arg("--history-file")
        .arg(dir.path().join("history.json"));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_1.txt").and(predicate::str::contains("file_1.txt")));

    assert!(file1.exists());
    assert!(!dir.path().join("file_1.txt").exists());
}

#[test]
fn test_cli_quiet_mode() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_1.txt");
    File::create(&file1).unwrap();

    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("test_(\\d+).txt")
        .arg("-p")
        .arg("file_${1}.txt")
        .arg("-q")
        .arg("--history-file")
        .arg(dir.path().join("history.json"));

    cmd.assert().success().stdout(predicate::str::is_empty());

    assert!(dir.path().join("file_1.txt").exists());
}

#[test]
fn test_cli_invalid_regex() {
    let dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("test_(\\d+.txt") // Missing closing parenthesis
        .arg("-p")
        .arg("file_${1}.txt")
        .arg("--history-file")
        .arg(dir.path().join("history.json"));

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid regex"));
}

#[test]
fn test_cli_not_a_directory() {
    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg("non_existent_directory_12345")
        .arg("-r")
        .arg("foo")
        .arg("-p")
        .arg("bar")
        .arg("--history-file")
        .arg("history.json"); // This one is tricky as dir is not defined here, but the command is expected to fail anyway due to not a dir.

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("path is not a directory"));
}

#[test]
fn test_cli_undo() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_1.txt");
    File::create(&file1).unwrap();
    let history_file = dir.path().join("history.json");

    // 1. Rename
    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("test_(\\d+).txt")
        .arg("-p")
        .arg("file_${1}.txt")
        .arg("--history-file")
        .arg(&history_file);

    cmd.assert().success();
    assert!(dir.path().join("file_1.txt").exists());
    assert!(!file1.exists());
    assert!(history_file.exists());

    // 2. Undo
    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("--undo")
        .arg("--history-file")
        .arg(&history_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("file_1.txt").and(predicate::str::contains("test_1.txt")));

    assert!(file1.exists());
    assert!(!dir.path().join("file_1.txt").exists());
}

#[test]
fn test_cli_collision_skip() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_1.txt");
    let existing = dir.path().join("file_1.txt");
    File::create(&file1).unwrap();
    File::create(&existing).unwrap();

    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("test_(\\d+).txt")
        .arg("-p")
        .arg("file_${1}.txt")
        .arg("-c")
        .arg("skip")
        .arg("--history-file")
        .arg(dir.path().join("history.json"));

    cmd.assert().success();

    assert!(file1.exists()); // Should be skipped
    assert!(existing.exists());
}

#[test]
fn test_cli_collision_overwrite() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_1.txt");
    let existing = dir.path().join("file_1.txt");
    File::create(&file1).unwrap();
    std::fs::write(&existing, "old content").unwrap();

    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("test_(\\d+).txt")
        .arg("-p")
        .arg("file_${1}.txt")
        .arg("-c")
        .arg("overwrite")
        .arg("--history-file")
        .arg(dir.path().join("history.json"));

    cmd.assert().success();

    assert!(!file1.exists());
    assert!(existing.exists());
}

#[test]
fn test_cli_collision_suffix() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_1.txt");
    let existing = dir.path().join("file_1.txt");
    File::create(&file1).unwrap();
    File::create(&existing).unwrap();

    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("test_(\\d+).txt")
        .arg("-p")
        .arg("file_${1}.txt")
        .arg("-c")
        .arg("suffix")
        .arg("--history-file")
        .arg(dir.path().join("history.json"));

    cmd.assert().success();

    assert!(!file1.exists());
    assert!(existing.exists());
    assert!(dir.path().join("file_1 (1).txt").exists());
}

#[test]
fn test_cli_undo_no_history() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("--undo")
        .arg("--history-file")
        .arg(dir.path().join("non_existent_history.json"));

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));
}

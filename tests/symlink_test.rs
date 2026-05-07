#![cfg(unix)]

use assert_cmd::Command;
use std::fs::File;
use tempfile::tempdir;

#[test]
fn test_cli_symlink_ignore() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file.txt");
    File::create(&file).unwrap();
    let link = dir.path().join("link.txt");
    std::os::unix::fs::symlink(&file, &link).unwrap();

    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("link")
        .arg("-p")
        .arg("new_link")
        .arg("--symlinks")
        .arg("ignore");

    cmd.assert().success();

    assert!(link.exists());
    assert!(!dir.path().join("new_link.txt").exists());
}

#[test]
fn test_cli_symlink_rename() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file.txt");
    File::create(&file).unwrap();
    let link = dir.path().join("link.txt");
    std::os::unix::fs::symlink(&file, &link).unwrap();

    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("link")
        .arg("-p")
        .arg("new_link")
        .arg("--symlinks")
        .arg("rename");

    cmd.assert().success();

    assert!(!link.exists());
    assert!(dir.path().join("new_link.txt").exists());
    // Ensure it's still a symlink
    assert!(std::fs::symlink_metadata(dir.path().join("new_link.txt"))
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
fn test_cli_symlink_follow() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file.txt");
    File::create(&file).unwrap();
    let link = dir.path().join("link.txt");
    std::os::unix::fs::symlink(&file, &link).unwrap();

    let mut cmd = Command::cargo_bin("bmv").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("-r")
        .arg("file")
        .arg("-p")
        .arg("new_file")
        .arg("--symlinks")
        .arg("follow");

    cmd.assert().success();

    assert!(!file.exists());
    assert!(dir.path().join("new_file.txt").exists());
    assert!(std::fs::symlink_metadata(&link).is_ok());
    // link.txt still exists but points to the old name (file.txt), so it might be broken.
    // That's the expected behavior of "follow and rename target".
}

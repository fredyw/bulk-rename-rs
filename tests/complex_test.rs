extern crate bmv;

use bmv::{BulkRename, NoOpCallback};
use std::fs;
use std::fs::File;
use tempfile::tempdir;

#[test]
fn test_complex_recursive_rename() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a complex structure
    // root/
    //   file1.txt
    //   subdir1/
    //     file2.txt
    //     subdir2/
    //       file3.txt
    //   subdir3/
    //     other.dat

    fs::create_dir_all(root.join("subdir1/subdir2")).unwrap();
    fs::create_dir_all(root.join("subdir3")).unwrap();

    File::create(root.join("file1.txt")).unwrap();
    File::create(root.join("subdir1/file2.txt")).unwrap();
    File::create(root.join("subdir1/subdir2/file3.txt")).unwrap();
    File::create(root.join("subdir3/other.dat")).unwrap();

    let bulk_rename = BulkRename::new(root, r"file(\d)\.txt", r"renamed_$1.txt").unwrap();
    bulk_rename.execute(NoOpCallback::new());

    // Check results
    assert!(root.join("renamed_1.txt").exists());
    assert!(root.join("subdir1/renamed_2.txt").exists());
    assert!(root.join("subdir1/subdir2/renamed_3.txt").exists());
    assert!(root.join("subdir3/other.dat").exists());

    assert!(!root.join("file1.txt").exists());
    assert!(!root.join("subdir1/file2.txt").exists());
    assert!(!root.join("subdir1/subdir2/file3.txt").exists());
}

#[test]
fn test_mixed_matches() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    File::create(root.join("match1.txt")).unwrap();
    File::create(root.join("no_match.txt")).unwrap();
    File::create(root.join("match2.txt")).unwrap();

    let bulk_rename = BulkRename::new(root, r"match(\d)\.txt", r"ok_$1.txt").unwrap();
    bulk_rename.execute(NoOpCallback::new());

    assert!(root.join("ok_1.txt").exists());
    assert!(root.join("ok_2.txt").exists());
    assert!(root.join("no_match.txt").exists());

    assert!(!root.join("match1.txt").exists());
    assert!(!root.join("match2.txt").exists());
}

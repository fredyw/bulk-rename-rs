extern crate bmv;

use bmv::{BulkRename, CollisionStrategy, NoOpCallback};
use std::fs;
use std::fs::File;
use std::path::Path;

#[test]
fn collision_skip() {
    let tmp_path = Path::new("tmp_skip");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_skip/file1.txt").unwrap();
    File::create("tmp_skip/file2.txt").unwrap();
    File::create("tmp_skip/target.txt").unwrap();

    // Try to rename file1.txt and file2.txt to target.txt
    let bulk_rename = BulkRename::new(tmp_path, r"file\d.txt", "target.txt")
        .unwrap()
        .with_collision_strategy(CollisionStrategy::Skip);
    bulk_rename.execute(NoOpCallback::new());

    // Both should be skipped because target.txt already exists
    assert!(Path::new("tmp_skip/file1.txt").exists());
    assert!(Path::new("tmp_skip/file2.txt").exists());
    assert!(Path::new("tmp_skip/target.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn collision_overwrite() {
    let tmp_path = Path::new("tmp_overwrite");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_overwrite/file1.txt").unwrap();
    File::create("tmp_overwrite/target.txt").unwrap();

    fs::write("tmp_overwrite/file1.txt", "new content").unwrap();
    fs::write("tmp_overwrite/target.txt", "old content").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"file1.txt", "target.txt")
        .unwrap()
        .with_collision_strategy(CollisionStrategy::Overwrite);
    bulk_rename.execute(NoOpCallback::new());

    // file1.txt should be gone, target.txt should have new content
    assert!(!Path::new("tmp_overwrite/file1.txt").exists());
    assert!(Path::new("tmp_overwrite/target.txt").exists());
    assert_eq!(
        fs::read_to_string("tmp_overwrite/target.txt").unwrap(),
        "new content"
    );

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn collision_suffix() {
    let tmp_path = Path::new("tmp_suffix");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_suffix/file1.txt").unwrap();
    File::create("tmp_suffix/file2.txt").unwrap();
    File::create("tmp_suffix/target.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"file(\d).txt", "target.txt")
        .unwrap()
        .with_collision_strategy(CollisionStrategy::Suffix);
    bulk_rename.execute(NoOpCallback::new());

    // One should be target (1).txt, another target (2).txt (order not guaranteed due to parallelism)
    assert!(Path::new("tmp_suffix/target.txt").exists());
    assert!(Path::new("tmp_suffix/target (1).txt").exists());
    assert!(Path::new("tmp_suffix/target (2).txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn internal_collision_skip() {
    let tmp_path = Path::new("tmp_internal_skip");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_internal_skip/file1.txt").unwrap();
    File::create("tmp_internal_skip/file2.txt").unwrap();

    // Both map to target.txt. One should win, one should be skipped.
    let bulk_rename = BulkRename::new(tmp_path, r"file\d.txt", "target.txt")
        .unwrap()
        .with_collision_strategy(CollisionStrategy::Skip);
    bulk_rename.execute(NoOpCallback::new());

    assert!(Path::new("tmp_internal_skip/target.txt").exists());
    // Exactly one of file1 or file2 should still exist
    let f1 = Path::new("tmp_internal_skip/file1.txt").exists();
    let f2 = Path::new("tmp_internal_skip/file2.txt").exists();
    assert!(f1 ^ f2);

    fs::remove_dir_all(tmp_path).unwrap();
}

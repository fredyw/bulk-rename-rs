use bulk_rename_rs::{BulkRename, CollisionStrategy, HistoryCallback, NoOpCallback, RenameHistory};
use std::fs::File;
use std::sync::Mutex;
use tempfile::tempdir;

#[test]
fn test_history_collection() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let f1 = root.join("file1.txt");
    let f2 = root.join("file2.txt");
    File::create(&f1).unwrap();
    File::create(&f2).unwrap();

    let bulk_rename = BulkRename::new(root, r"file(\d)\.txt", r"renamed_$1.txt")
        .unwrap()
        .with_collision_strategy(CollisionStrategy::Skip);

    let history_records = Mutex::new(Vec::new());
    let callback = HistoryCallback::new(NoOpCallback::new(), &history_records);

    bulk_rename.execute(callback);

    let records = history_records.into_inner().unwrap();
    assert_eq!(records.len(), 2);

    let r1 = records.iter().find(|r| r.old_path == f1).unwrap();
    assert_eq!(r1.new_path, root.join("renamed_1.txt"));

    let r2 = records.iter().find(|r| r.old_path == f2).unwrap();
    assert_eq!(r2.new_path, root.join("renamed_2.txt"));
}

#[test]
fn test_bulk_rename_undo() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let f1 = root.join("file1.txt");
    let f2 = root.join("file2.txt");
    File::create(&f1).unwrap();
    File::create(&f2).unwrap();

    let bulk_rename = BulkRename::new(root, r"file(\d)\.txt", r"renamed_$1.txt")
        .unwrap()
        .with_collision_strategy(CollisionStrategy::Skip);

    let history_records = Mutex::new(Vec::new());
    let callback = HistoryCallback::new(NoOpCallback::new(), &history_records);

    // Perform rename
    bulk_rename.execute(callback);

    let renamed1 = root.join("renamed_1.txt");
    let renamed2 = root.join("renamed_2.txt");
    assert!(renamed1.exists());
    assert!(renamed2.exists());
    assert!(!f1.exists());
    assert!(!f2.exists());

    let history = RenameHistory {
        records: history_records.into_inner().unwrap(),
    };

    // Perform undo
    BulkRename::undo(&history, NoOpCallback::new());

    assert!(!renamed1.exists());
    assert!(!renamed2.exists());
    assert!(f1.exists());
    assert!(f2.exists());
}

#[test]
fn test_undo_with_collision_suffix() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Setup: file1.txt and file2.txt both map to target.txt
    // With Suffix strategy: file1.txt -> target.txt, file2.txt -> target (1).txt
    let f1 = root.join("file1.txt");
    let f2 = root.join("file2.txt");
    File::create(&f1).unwrap();
    File::create(&f2).unwrap();

    let bulk_rename = BulkRename::new(root, r"file\d\.txt", "target.txt")
        .unwrap()
        .with_collision_strategy(CollisionStrategy::Suffix);

    let history_records = Mutex::new(Vec::new());
    let callback = HistoryCallback::new(NoOpCallback::new(), &history_records);

    bulk_rename.execute(callback);

    let target = root.join("target.txt");
    let target_suffix = root.join("target (1).txt");
    assert!(target.exists());
    assert!(target_suffix.exists());

    let history = RenameHistory {
        records: history_records.into_inner().unwrap(),
    };

    // Perform undo
    BulkRename::undo(&history, NoOpCallback::new());

    assert!(!target.exists());
    assert!(!target_suffix.exists());
    assert!(f1.exists());
    assert!(f2.exists());
}

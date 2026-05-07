extern crate bulk_rename_rs;

use bulk_rename_rs::{BulkRename, CollisionStrategy, NoOpCallback, TransactionStrategy};
use std::fs;
use std::fs::File;
use std::path::Path;

#[test]
fn test_transaction_continue() {
    let tmp_path = Path::new("tmp_tx_continue");
    if tmp_path.exists() {
        fs::remove_dir_all(tmp_path).unwrap();
    }
    fs::create_dir_all(tmp_path).unwrap();
    File::create(tmp_path.join("file1.txt")).unwrap();
    File::create(tmp_path.join("file2.txt")).unwrap();

    // To make it fail, we can try to rename one to a path that is a directory
    fs::create_dir(tmp_path.join("success2.txt")).unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"file(\d)\.txt", r"success$1.txt")
        .unwrap()
        .with_transaction_strategy(TransactionStrategy::Continue)
        .with_collision_strategy(CollisionStrategy::Overwrite);

    // Let's use a custom callback to track results
    struct TestCallback {
        pub ok_count: usize,
        pub err_count: usize,
    }
    impl bulk_rename_rs::Callback for TestCallback {
        fn on_ok(&mut self, _: &Path, _: &Path) {
            self.ok_count += 1;
        }
        fn on_error(&mut self, _: &Path, _: &Path, _: std::io::Error) {
            self.err_count += 1;
        }

        fn on_rollback_ok(&mut self, _: &Path, _: &Path) {}

        fn on_rollback_error(&mut self, _: &Path, _: &Path, _: std::io::Error) {}
    }

    let mut cb = TestCallback {
        ok_count: 0,
        err_count: 0,
    };

    bulk_rename.execute(&mut cb);

    assert_eq!(cb.ok_count, 1);
    assert_eq!(cb.err_count, 1);

    assert!(tmp_path.join("success1.txt").exists());
    assert!(!tmp_path.join("file1.txt").exists());
    assert!(tmp_path.join("file2.txt").exists()); // Failed to rename to success2.txt because it's a dir

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn test_transaction_abort() {
    let tmp_path = Path::new("tmp_tx_abort");
    if tmp_path.exists() {
        fs::remove_dir_all(tmp_path).unwrap();
    }
    fs::create_dir_all(tmp_path).unwrap();
    File::create(tmp_path.join("file1.txt")).unwrap();
    File::create(tmp_path.join("file2.txt")).unwrap();
    File::create(tmp_path.join("file3.txt")).unwrap();

    // Force file1.txt to fail
    fs::create_dir(tmp_path.join("success1.txt")).unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"file(\d)\.txt", r"success$1.txt")
        .unwrap()
        .with_transaction_strategy(TransactionStrategy::Abort)
        .with_collision_strategy(CollisionStrategy::Overwrite);

    bulk_rename.execute(NoOpCallback::new());

    // file1 fails, so file2 and file3 should NOT be renamed
    assert!(tmp_path.join("file2.txt").exists());
    assert!(tmp_path.join("file3.txt").exists());
    assert!(!tmp_path.join("success2.txt").exists());
    assert!(!tmp_path.join("success3.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn test_transaction_rollback() {
    let tmp_path = Path::new("tmp_tx_rollback");
    if tmp_path.exists() {
        fs::remove_dir_all(tmp_path).unwrap();
    }
    fs::create_dir_all(tmp_path).unwrap();
    File::create(tmp_path.join("file1.txt")).unwrap();
    File::create(tmp_path.join("file2.txt")).unwrap();
    File::create(tmp_path.join("file3.txt")).unwrap();

    // Force file3.txt to fail.
    // Plan is sorted by path usually, so file1, file2, file3.
    fs::create_dir(tmp_path.join("success3.txt")).unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"file(\d)\.txt", r"success$1.txt")
        .unwrap()
        .with_transaction_strategy(TransactionStrategy::Rollback)
        .with_collision_strategy(CollisionStrategy::Overwrite);

    bulk_rename.execute(NoOpCallback::new());

    // file1 and file2 are renamed first (sequentially), then file3 fails.
    // Rollback should move success1 back to file1 and success2 back to file2.
    assert!(
        tmp_path.join("file1.txt").exists(),
        "file1.txt should have been rolled back"
    );
    assert!(
        tmp_path.join("file2.txt").exists(),
        "file2.txt should have been rolled back"
    );
    assert!(
        tmp_path.join("file3.txt").exists(),
        "file3.txt should have stayed (failed to rename)"
    );
    assert!(!tmp_path.join("success1.txt").exists());
    assert!(!tmp_path.join("success2.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

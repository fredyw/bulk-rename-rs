extern crate bmv;

use bmv::{BulkRename, Error, NoOpCallback};
use std::fs;
use std::fs::File;
use std::path::Path;

#[test]
fn bulk_rename_has_match() {
    let tmp_path = Path::new("tmp1");
    fs::create_dir_all("tmp1/a/b/c/").unwrap();
    File::create("tmp1/test_123.txt").unwrap();
    File::create("tmp1/foo_123.txt").unwrap();
    File::create("tmp1/a/test_234.txt").unwrap();
    File::create("tmp1/a/foo_234.txt").unwrap();
    File::create("tmp1/a/b/test_345.txt").unwrap();
    File::create("tmp1/a/b/foo_345.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"(test)_(\d+).txt", r"${2}_${1}.txt").unwrap();
    bulk_rename.bulk_rename(NoOpCallback::new());

    assert!(Path::new("tmp1/foo_123.txt").exists());
    assert!(Path::new("tmp1/a/foo_234.txt").exists());
    assert!(Path::new("tmp1/a/b/foo_345.txt").exists());

    assert!(!Path::new("tmp1/test_123.txt").exists());
    assert!(!Path::new("tmp1/a/test_234.txt").exists());
    assert!(!Path::new("tmp1/a/b/test_345.txt").exists());

    assert!(Path::new("tmp1/123_test.txt").exists());
    assert!(Path::new("tmp1/a/234_test.txt").exists());
    assert!(Path::new("tmp1/a/b/345_test.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn bulk_rename_no_match() {
    let tmp_path = Path::new("tmp2");
    fs::create_dir_all("tmp2/a/b/c/").unwrap();
    File::create("tmp2/test_123.txt").unwrap();
    File::create("tmp2/a/test_234.txt").unwrap();
    File::create("tmp2/a/b/test_345.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"foobar.txt", r"${2}_${1}.txt").unwrap();
    bulk_rename.bulk_rename(NoOpCallback::new());

    assert!(Path::new("tmp2/test_123.txt").exists());
    assert!(Path::new("tmp2/a/test_234.txt").exists());
    assert!(Path::new("tmp2/a/b/test_345.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn bulk_rename_unicode_chars() {
    let tmp_path = Path::new("tmp3");
    fs::create_dir_all("tmp3").unwrap();
    File::create("tmp3/中文.txt").unwrap();
    File::create("tmp3/日本.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"中文.txt", r"英语.txt").unwrap();
    bulk_rename.bulk_rename(NoOpCallback::new());

    assert!(Path::new("tmp3/日本.txt").exists());
    assert!(!Path::new("tmp3/中文.txt").exists());
    assert!(Path::new("tmp3/英语.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn path_is_not_a_directory() {
    let bulk_rename = BulkRename::new(Path::new("doesntexist"), "foo", "bar");
    assert!(matches!(bulk_rename.unwrap_err(), Error::NotDirError));
}

#[test]
fn regex_is_invalid() {
    let bulk_rename = BulkRename::new(Path::new("."), r"(\d+", "bar");
    assert!(matches!(bulk_rename.unwrap_err(), Error::RegexError(_)));
}

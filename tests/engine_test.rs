extern crate bmv;

use bmv::{BulkRename, Error, NoOpCallback};
use regex::Regex;
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
    bulk_rename.execute(NoOpCallback::new());

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
    bulk_rename.execute(NoOpCallback::new());

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
    bulk_rename.execute(NoOpCallback::new());

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

#[test]
fn bulk_rename_ignore_case() {
    let tmp_path = Path::new("tmp_case");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_case/TEST.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"test", "renamed")
        .unwrap()
        .with_case_insensitive(true)
        .unwrap();
    bulk_rename.execute(NoOpCallback::new());

    assert!(Path::new("tmp_case/renamed.txt").exists());
    assert!(!Path::new("tmp_case/TEST.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn bulk_rename_extension_filter() {
    let tmp_path = Path::new("tmp_ext");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_ext/test.txt").unwrap();
    File::create("tmp_ext/test.jpg").unwrap();

    let mut exts = std::collections::HashSet::new();
    exts.insert("txt".to_string());

    let bulk_rename = BulkRename::new(tmp_path, r"test", "renamed")
        .unwrap()
        .with_extensions(exts);
    bulk_rename.execute(NoOpCallback::new());

    assert!(Path::new("tmp_ext/renamed.txt").exists());
    assert!(Path::new("tmp_ext/test.jpg").exists());
    assert!(!Path::new("tmp_ext/test.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn bulk_rename_include_exclude() {
    let tmp_path = Path::new("tmp_inc_exc");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_inc_exc/include_me.txt").unwrap();
    File::create("tmp_inc_exc/exclude_me.txt").unwrap();
    File::create("tmp_inc_exc/other.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"(.*)_me", r"renamed_$1")
        .unwrap()
        .with_include_patterns(vec![".*_me".to_string()])
        .unwrap()
        .with_exclude_patterns(vec!["exclude.*".to_string()])
        .unwrap();
    bulk_rename.execute(NoOpCallback::new());

    assert!(Path::new("tmp_inc_exc/renamed_include.txt").exists());
    assert!(Path::new("tmp_inc_exc/exclude_me.txt").exists());
    assert!(Path::new("tmp_inc_exc/other.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn bulk_rename_max_depth() {
    let tmp_path = Path::new("tmp_depth");
    fs::create_dir_all("tmp_depth/sub").unwrap();
    File::create("tmp_depth/root.txt").unwrap();
    File::create("tmp_depth/sub/nested.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"(.*)\.txt", r"renamed_$1.txt")
        .unwrap()
        .with_max_depth(Some(1));
    bulk_rename.execute(NoOpCallback::new());

    assert!(Path::new("tmp_depth/renamed_root.txt").exists());
    assert!(Path::new("tmp_depth/sub/nested.txt").exists());
    assert!(!Path::new("tmp_depth/root.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn bulk_rename_counter() {
    let tmp_path = Path::new("tmp_counter");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_counter/file_a.txt").unwrap();
    File::create("tmp_counter/file_b.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"file_(.*)\.txt", r"image_{i}.txt")
        .unwrap()
        .with_counter_start(10);
    // Use sequential to ensure deterministic order for test assertion
    bulk_rename.run_seq(|old, new| {
        fs::rename(old, new).unwrap();
    });

    assert!(Path::new("tmp_counter/image_10.txt").exists());
    assert!(Path::new("tmp_counter/image_11.txt").exists());

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn bulk_rename_counter_padding() {
    let tmp_path = Path::new("tmp_counter_pad");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_counter_pad/file.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"file\.txt", r"file_{i:3}.txt").unwrap();

    // Let's do real renames.
    let mut names = Vec::new();
    bulk_rename.run_seq(|_, new| {
        names.push(new.file_name().unwrap().to_string_lossy().to_string());
    });

    assert!(names.contains(&"file_001.txt".to_string()));

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn bulk_rename_date() {
    let tmp_path = Path::new("tmp_date");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_date/file.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"file\.txt", r"file_{date}.txt").unwrap();

    let mut names = Vec::new();
    bulk_rename.run_seq(|_, new| {
        names.push(new.file_name().unwrap().to_string_lossy().to_string());
    });

    let name = &names[0];
    let re = Regex::new(r"file_\d{4}-\d{2}-\d{2}\.txt").unwrap();
    assert!(
        re.is_match(name),
        "Name {} did not match expected date format",
        name
    );

    fs::remove_dir_all(tmp_path).unwrap();
}

#[test]
fn bulk_rename_date_custom() {
    let tmp_path = Path::new("tmp_date_custom");
    fs::create_dir_all(tmp_path).unwrap();
    File::create("tmp_date_custom/file.txt").unwrap();

    let bulk_rename = BulkRename::new(tmp_path, r"file\.txt", r"file_{date:%Y}.txt").unwrap();

    let mut names = Vec::new();
    bulk_rename.run_seq(|_, new| {
        names.push(new.file_name().unwrap().to_string_lossy().to_string());
    });

    let name = &names[0];
    let current_year = chrono::Local::now().format("%Y").to_string();
    assert_eq!(name, &format!("file_{}.txt", current_year));

    fs::remove_dir_all(tmp_path).unwrap();
}

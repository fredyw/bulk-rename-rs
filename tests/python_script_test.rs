use assert_cmd::Command;
use std::fs::File;
use tempfile::tempdir;

#[test]
fn test_python_script_inline() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_file.txt");
    File::create(&file1).unwrap();

    let mut cmd = Command::cargo_bin("bren").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("--python-script")
        .arg("result = name.upper()");

    cmd.assert().success();

    assert!(dir.path().join("TEST_FILE.TXT").exists());
}

#[test]
fn test_python_file() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("data.txt");
    File::create(&file1).unwrap();

    let script_file = dir.path().join("script.py");
    std::fs::write(&script_file, "result = name.replace('.txt', '.csv')").unwrap();

    let mut cmd = Command::cargo_bin("bren").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("--python-file")
        .arg(&script_file);

    cmd.assert().success();

    assert!(dir.path().join("data.csv").exists());
}

#[test]
fn test_python_import_re() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("file123.txt");
    File::create(&file1).unwrap();

    let mut cmd = Command::cargo_bin("bren").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("--python-script")
        .arg("import re; result = re.sub(r'\\d+', '', name)");

    cmd.assert().success();

    assert!(dir.path().join("file.txt").exists());
}

#[test]
fn test_python_missing_result() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_file.txt");
    File::create(&file1).unwrap();

    let mut cmd = Command::cargo_bin("bren").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("--python-script")
        .arg("x = 1");

    cmd.assert()
        .success() // The tool currently continues on script error but prints to stderr
        .stderr(predicates::str::contains(
            "Python script must set the 'result' variable to the new filename.",
        ));
}

#[test]
fn test_python_wrong_type() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_file.txt");
    File::create(&file1).unwrap();

    let mut cmd = Command::cargo_bin("bren").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("--python-script")
        .arg("result = 123");

    cmd.assert()
        .success()
        .stderr(predicates::str::contains(
            "The 'result' variable must be a string, but found type 'int'.",
        ));
}

#[test]
fn test_python_runtime_error() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_file.txt");
    File::create(&file1).unwrap();

    let mut cmd = Command::cargo_bin("bren").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("--python-script")
        .arg("1 / 0");

    cmd.assert()
        .success()
        .stderr(predicates::str::contains("division by zero"));
}

#[test]
fn test_python_compile_error() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test_file.txt");
    File::create(&file1).unwrap();

    let mut cmd = Command::cargo_bin("bren").unwrap();
    cmd.arg("-f")
        .arg(dir.path())
        .arg("--python-script")
        .arg("if True");

    cmd.assert()
        .success()
        .stderr(predicates::str::contains("invalid syntax"));
}

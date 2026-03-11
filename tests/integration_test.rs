use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

mod common;

/// Build a Command for the reg-rs binary with the test data dir set
fn reg_rs() -> Command {
    let mut cmd = Command::cargo_bin("reg-rs").unwrap();
    cmd.env("REG_RS_DATA_DIR", common::test_data_dir());
    cmd
}

#[test]
fn integration_test_reg_rs_help() {
    common::setup();
    reg_rs()
        .arg("-h")
        .assert()
        .success()
        .stderr("")
        .stdout(predicate::str::contains(
            "Usage: reg-rs [OPTIONS] <COMMAND>",
        ))
        .stdout(predicate::str::contains(
            "create  Creates a new test of a specified command",
        ))
        .stdout(predicate::str::contains(
            "remove  Removes previously created test",
        ))
        .stdout(predicate::str::contains("report  Reports counts/summary"))
        .stdout(predicate::str::contains("run     Runs a test"))
        .stdout(predicate::str::contains(
            "status  Starts a server to monitor",
        ))
        .stdout(predicate::str::contains("-d, --debug"))
        .stdout(predicate::str::contains("-l, --logging"))
        .stdout(predicate::str::contains("-h, --help"))
        .stdout(predicate::str::contains("-V, --version"));
}

#[test]
fn integration_test_version() {
    common::setup();
    reg_rs()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("reg-rs "));
}

#[test]
fn integration_test_create_help() {
    common::setup();
    reg_rs()
        .args(["create", "-h"])
        .assert()
        .success()
        .stderr("")
        .stdout(predicate::str::contains(
            "Creates a new test of a specified command",
        ))
        .stdout(predicate::str::contains("-t, --test"))
        .stdout(predicate::str::contains("-c, --command"));
}

#[test]
fn integration_test_run_help() {
    common::setup();
    reg_rs()
        .args(["run", "-h"])
        .assert()
        .success()
        .stderr("")
        .stdout(predicate::str::contains("Runs a test"))
        .stdout(predicate::str::contains("-p, --pattern"))
        .stdout(predicate::str::contains("-n, --dry-run"));
}

#[test]
fn integration_test_report_help() {
    common::setup();
    reg_rs()
        .args(["report", "-h"])
        .assert()
        .success()
        .stderr("")
        .stdout(predicate::str::contains("Reports counts/summary"))
        .stdout(predicate::str::contains("-p, --pattern"))
        .stdout(predicate::str::contains("-v"));
}

#[test]
fn integration_test_remove_help() {
    common::setup();
    reg_rs()
        .args(["remove", "-h"])
        .assert()
        .success()
        .stderr("")
        .stdout(predicate::str::contains("Removes previously created test"))
        .stdout(predicate::str::contains("-p, --pattern"));
}

#[test]
fn integration_test_status_help() {
    common::setup();
    reg_rs()
        .args(["status", "-h"])
        .assert()
        .success()
        .stderr("")
        .stdout(predicate::str::contains("Starts a server to monitor"))
        .stdout(predicate::str::contains("-p, --pattern"))
        .stdout(predicate::str::contains("-l, --localhost-port"));
}

#[test]
fn integration_test_create_and_run() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("integration_test.tdb");
    let test_pattern = "integration_test";

    // Clean up any existing test file (including lock file)
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));

    // Create a new test
    reg_rs()
        .args(["create", "-t", "integration_test", "-c", "echo hello"])
        .assert()
        .success();

    assert!(
        test_db.exists(),
        "Test database should exist at {}",
        test_db.display()
    );

    // Run the test
    reg_rs()
        .args(["run", "-p", test_pattern])
        .assert()
        .success();

    // Report on the test
    reg_rs()
        .args(["report", "-p", test_pattern])
        .assert()
        .success()
        .stdout(predicate::str::contains("matched pattern"));

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));
}

#[test]
fn integration_test_create_run_and_remove() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("remove_test.tdb");
    let test_pattern = "remove_test";

    // Clean up any leftover files
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));

    // Create a test
    reg_rs()
        .args(["create", "-t", "remove_test", "-c", "echo remove me"])
        .assert()
        .success();

    assert!(test_db.exists(), "Test database should exist");

    // Remove the test
    reg_rs()
        .args(["remove", "-p", test_pattern])
        .assert()
        .success();

    assert!(
        !test_db.exists(),
        "Test database should be removed at {}",
        test_db.display()
    );

    let lock_file = format!("{}.lock", test_db.display());
    assert!(
        !std::path::Path::new(&lock_file).exists(),
        "Lock file should be removed"
    );
}

#[test]
fn integration_test_run_no_matching_tests() {
    common::setup();
    reg_rs()
        .args(["run", "-p", "nonexistent_pattern_xyz"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "warning: no tests matched pattern",
        ));
}

#[test]
fn integration_test_report_no_matching_tests() {
    common::setup();
    reg_rs()
        .args(["report", "-p", "nonexistent_pattern_xyz"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "warning: no tests matched pattern",
        ));
}

#[test]
fn integration_test_remove_no_matching_tests() {
    common::setup();
    reg_rs()
        .args(["remove", "-p", "nonexistent_pattern_xyz"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "warning: no tests matched pattern",
        ));
}

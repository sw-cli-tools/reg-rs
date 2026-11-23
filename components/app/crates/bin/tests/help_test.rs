//! Help output and version for each subcommand.

use predicates::prelude::*;

mod common;
use common::reg_rs;

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
        .stdout(predicate::str::contains("analyze"))
        .stdout(predicate::str::contains("Create a new test"))
        .stdout(predicate::str::contains("Lists all tests matching"))
        .stdout(predicate::str::contains("Shows detailed information"))
        .stdout(predicate::str::contains(
            "Converts existing .tdb test databases",
        ))
        .stdout(predicate::str::contains("Accepts the latest test output"))
        .stdout(predicate::str::contains("Clears latest run results"))
        .stdout(predicate::str::contains("Outputs test names for shell"))
        .stdout(predicate::str::contains(
            "Removes test database files matching",
        ))
        .stdout(predicate::str::contains("Reports on test results"))
        .stdout(predicate::str::contains("Runs previously created tests"))
        .stdout(predicate::str::contains("Starts a web server to monitor"))
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
        .stdout(predicate::str::contains("Create a new test"))
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
        .stdout(predicate::str::contains("Runs previously created tests"))
        .stdout(predicate::str::contains("-p, --pattern"))
        .stdout(predicate::str::contains("-n, --dry-run"))
        .stdout(predicate::str::contains("--parallel"));
}

#[test]
fn integration_test_run_parallel_no_matching_tests() {
    common::setup();
    reg_rs()
        .args(["run", "-p", "nonexistent_pattern_xyz", "--parallel"])
        .assert()
        .success();
}

#[test]
fn integration_test_report_help() {
    common::setup();
    reg_rs()
        .args(["report", "-h"])
        .assert()
        .success()
        .stderr("")
        .stdout(predicate::str::contains("Reports on test results"))
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
        .stdout(predicate::str::contains(
            "Removes test database files matching",
        ))
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
        .stdout(predicate::str::contains("Starts a web server to monitor"))
        .stdout(predicate::str::contains("-p, --pattern"))
        .stdout(predicate::str::contains("-l, --localhost-port"));
}

#[test]
fn integration_test_create_help_shows_preprocess() {
    common::setup();
    reg_rs()
        .args(["create", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-P, --preprocess"));
}

#[test]
fn integration_test_create_help_shows_context() {
    common::setup();
    reg_rs()
        .args(["create", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-C, --context"));
}

#[test]
fn integration_test_create_help_shows_diff_mode() {
    common::setup();
    reg_rs()
        .args(["create", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-M, --diff-mode"));
}

#[test]
fn integration_test_create_help_shows_doc_metadata() {
    common::setup();
    reg_rs()
        .args(["create", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--desc"))
        .stdout(predicate::str::contains("--expects"))
        .stdout(predicate::str::contains("--flaky-note"));
}

//! Creating and running tests: preprocess, diff modes and doc metadata.

use std::fs;

use predicates::prelude::*;

mod common;
use common::reg_rs;

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
    let test_rgt = data_dir.join("remove_test.rgt");
    let test_out = data_dir.join("remove_test.out");
    let test_tdb = data_dir.join("remove_test.tdb");
    let test_pattern = "remove_test";

    // Clean up any leftover files
    let _ = fs::remove_file(&test_rgt);
    let _ = fs::remove_file(&test_out);
    let _ = fs::remove_file(&test_tdb);
    let _ = fs::remove_file(format!("{}.lock", test_tdb.display()));
    let _ = fs::remove_file(format!("{}.lock", test_rgt.display()));

    // Create a test — now produces .rgt + .out + .tdb cache
    reg_rs()
        .args(["create", "-t", "remove_test", "-c", "echo remove me"])
        .assert()
        .success();

    assert!(test_rgt.exists(), "Test .rgt should exist");
    assert!(test_out.exists(), "Test .out baseline should exist");
    assert!(test_tdb.exists(), "Test .tdb cache should exist");

    // Remove the test
    reg_rs()
        .args(["remove", "-p", test_pattern])
        .assert()
        .success();

    assert!(
        !test_rgt.exists(),
        "Test .rgt should be removed at {}",
        test_rgt.display()
    );
    assert!(!test_out.exists(), "Test .out should be removed");
    assert!(!test_tdb.exists(), "Test .tdb cache should be removed");
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

// --- Preprocess tests ---

#[test]
fn integration_test_create_with_preprocess() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_rgt = data_dir.join("pp_test.rgt");
    let test_out = data_dir.join("pp_test.out");
    let test_tdb = data_dir.join("pp_test.tdb");
    common::cleanup_test_files(&data_dir, "pp_test");

    // Create a test with a preprocess that sorts lines
    reg_rs()
        .args([
            "create",
            "-t",
            "pp_test",
            "-c",
            "printf 'banana\\napple\\ncherry\\n'",
            "-P",
            "sort",
        ])
        .assert()
        .success();

    // Recreate to ensure clean state
    common::cleanup_test_files(&data_dir, "pp_test");

    // Create baseline: unsorted fruit list
    reg_rs()
        .args([
            "create",
            "-t",
            "pp_test",
            "-c",
            "printf 'banana\\napple\\ncherry\\n'",
            "-P",
            "sort",
        ])
        .assert()
        .success();

    assert!(test_rgt.exists(), ".rgt should exist");
    assert!(test_out.exists(), ".out should exist");
    assert!(test_tdb.exists(), ".tdb cache should exist");

    // Run and report — same command should pass
    reg_rs().args(["run", "-p", "pp_test"]).assert().success();

    reg_rs()
        .args(["report", "-p", "pp_test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("00001 passed"));

    // Clean up
    reg_rs()
        .args(["remove", "-p", "pp_test"])
        .assert()
        .success();
}

#[test]
fn integration_test_create_with_diff_mode_json() {
    common::setup();

    let data_dir = common::test_data_dir();
    common::cleanup_test_files(&data_dir, "json_test");

    // Create with JSON diff mode — keys in one order
    reg_rs()
        .args([
            "create",
            "-t",
            "json_test",
            "-c",
            r#"printf '{"z":1,"a":2,"m":3}'"#,
            "-M",
            "json",
        ])
        .assert()
        .success();

    assert!(data_dir.join("json_test.rgt").exists());

    // Run — same command produces same JSON, should pass after normalization
    reg_rs().args(["run", "-p", "json_test"]).assert().success();

    reg_rs()
        .args(["report", "-p", "json_test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("00001 passed"));

    // Clean up
    reg_rs()
        .args(["remove", "-p", "json_test"])
        .assert()
        .success();
}

#[test]
fn integration_test_create_with_diff_mode_lines_unordered() {
    common::setup();

    let data_dir = common::test_data_dir();
    common::cleanup_test_files(&data_dir, "lines_test");

    // Create with lines-unordered — output lines in one order
    reg_rs()
        .args([
            "create",
            "-t",
            "lines_test",
            "-c",
            "printf 'cherry\\napple\\nbanana\\n'",
            "-M",
            "lines-unordered",
        ])
        .assert()
        .success();

    assert!(data_dir.join("lines_test.rgt").exists());

    // Run — same lines, should pass after sorting
    reg_rs()
        .args(["run", "-p", "lines_test"])
        .assert()
        .success();

    reg_rs()
        .args(["report", "-p", "lines_test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("00001 passed"));

    // Clean up
    reg_rs()
        .args(["remove", "-p", "lines_test"])
        .assert()
        .success();
}

#[test]
fn integration_test_create_with_doc_metadata() {
    common::setup();

    let data_dir = common::test_data_dir();
    common::cleanup_test_files(&data_dir, "doc_meta_test");

    reg_rs()
        .args([
            "create",
            "-t",
            "doc_meta_test",
            "-c",
            "echo hello",
            "--desc",
            "Verifies echo works",
            "--expects",
            "Prints hello to stdout",
            "--flaky-note",
            "None - fully deterministic",
        ])
        .assert()
        .success();

    assert!(data_dir.join("doc_meta_test.rgt").exists());

    // Clean up
    reg_rs()
        .args(["remove", "-p", "doc_meta_test"])
        .assert()
        .success();
}

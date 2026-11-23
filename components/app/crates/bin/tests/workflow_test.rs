//! list, show, .rgt creation, rebase and exit codes.

use std::fs;

use predicates::prelude::*;

mod common;
use common::reg_rs;

#[test]
fn integration_test_list_tests() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("list_test.tdb");

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));

    // Create a test
    reg_rs()
        .args(["create", "-t", "list_test", "-c", "echo listing"])
        .assert()
        .success();

    // List should show pending (not yet run)
    reg_rs()
        .args(["list", "-p", "list_test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pending"))
        .stdout(predicate::str::contains("list_test"))
        .stdout(predicate::str::contains("1 test(s)"));

    // Run the test
    reg_rs().args(["run", "-p", "list_test"]).assert().success();

    // List should show PASS
    reg_rs()
        .args(["list", "-p", "list_test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"))
        .stdout(predicate::str::contains("list_test"));

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));
}

#[test]
fn integration_test_show_tests() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("show_test.tdb");

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));

    // Create a test with metadata
    reg_rs()
        .args([
            "create",
            "-t",
            "show_test",
            "-c",
            "echo showing",
            "--desc",
            "A test for show",
            "--timeout",
            "10",
        ])
        .assert()
        .success();

    // Show without verbosity — command and metadata
    reg_rs()
        .args(["show", "-p", "show_test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("show_test"))
        .stdout(predicate::str::contains("echo showing"))
        .stdout(predicate::str::contains("pending"))
        .stdout(predicate::str::contains("desc:"))
        .stdout(predicate::str::contains("A test for show"));

    // Show with -v — baseline output
    reg_rs()
        .args(["show", "-p", "show_test", "-v"])
        .assert()
        .success()
        .stdout(predicate::str::contains("baseline stdout"))
        .stdout(predicate::str::contains("showing"));

    // Run the test
    reg_rs().args(["run", "-p", "show_test"]).assert().success();

    // Show with -vv — latest results
    reg_rs()
        .args(["show", "-p", "show_test", "-vv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"))
        .stdout(predicate::str::contains("latest stdout"))
        .stdout(predicate::str::contains("latest exit: 0"));

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));
}

#[test]
fn integration_test_create_produces_rgt() {
    common::setup();

    let data_dir = common::test_data_dir();
    common::cleanup_test_files(&data_dir, "create_rgt_test");

    // Create a test with metadata — should produce .rgt directly
    reg_rs()
        .args([
            "create",
            "-t",
            "create_rgt_test",
            "-c",
            "echo hello_rgt",
            "--desc",
            "A direct rgt test",
        ])
        .assert()
        .success();

    let test_rgt = data_dir.join("create_rgt_test.rgt");
    let test_out = data_dir.join("create_rgt_test.out");
    let test_tdb = data_dir.join("create_rgt_test.tdb");

    // All three files should exist
    assert!(test_rgt.exists(), ".rgt should exist after create");
    assert!(test_out.exists(), ".out baseline should exist after create");
    assert!(test_tdb.exists(), ".tdb cache should exist after create");

    // .rgt should contain the command and metadata
    let rgt_content = fs::read_to_string(&test_rgt).expect("failed to read test file");
    assert!(
        rgt_content.contains("echo hello_rgt"),
        ".rgt should contain command"
    );
    assert!(
        rgt_content.contains("A direct rgt test"),
        ".rgt should contain desc"
    );

    // .out should contain the baseline stdout
    let out_content = fs::read_to_string(&test_out).expect("failed to read test file");
    assert_eq!(out_content, "hello_rgt\n");

    // Run should pass immediately (cache is populated)
    reg_rs()
        .args(["run", "-p", "create_rgt_test"])
        .assert()
        .success();

    // Clean up
    reg_rs()
        .args(["remove", "-p", "create_rgt_test"])
        .assert()
        .success();
}

#[test]
fn integration_test_rebase_rgt_test() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("rebase_test.tdb");
    let test_rgt = data_dir.join("rebase_test.rgt");
    let test_out = data_dir.join("rebase_test.out");

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));
    let _ = fs::remove_file(&test_rgt);
    let _ = fs::remove_file(&test_out);

    // Create .rgt test manually
    fs::write(&test_rgt, "command = \"echo rebased\"\n").expect("failed to write test file");
    fs::write(&test_out, "original output\n").expect("failed to write test file");

    // Run the .rgt test — will produce "rebased\n" which differs from "original output\n"
    // Exit code 1 = regressions detected (expected here)
    reg_rs().args(["run", "-p", "rebase_test"]).assert().code(1);

    // .tdb cache should exist now
    assert!(test_db.exists(), ".tdb cache should exist after run");

    // Show should indicate FAIL
    reg_rs()
        .args(["list", "-p", "rebase_test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("FAIL"));

    // Rebase — accept latest output
    reg_rs()
        .args(["rebase", "-p", "rebase_test"])
        .assert()
        .success()
        .stderr(predicate::str::contains("rebased:"))
        .stderr(predicate::str::contains("1 test(s) rebased"));

    // .out should now contain the new output
    let out_content = fs::read_to_string(&test_out).expect("failed to read test file");
    assert_eq!(out_content, "rebased\n");

    // Run again — should pass now
    reg_rs()
        .args(["run", "-p", "rebase_test"])
        .assert()
        .success();

    reg_rs()
        .args(["list", "-p", "rebase_test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"));

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));
    let _ = fs::remove_file(&test_rgt);
    let _ = fs::remove_file(&test_out);
}

/// Exit code 0 when all tests pass, 1 when regressions found, 2 on error.
#[test]
fn integration_test_exit_codes() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("exitcode_test.tdb");
    let test_rgt = data_dir.join("exitcode_test.rgt");
    let test_out = data_dir.join("exitcode_test.out");
    let lock_file = format!("{}.lock", test_db.display());

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(&lock_file);
    let _ = fs::remove_file(&test_rgt);
    let _ = fs::remove_file(&test_out);

    // Create and run — output matches baseline → exit 0
    reg_rs()
        .args(["create", "-t", "exitcode_test", "-c", "echo hello"])
        .assert()
        .code(0);
    reg_rs()
        .args(["run", "-p", "exitcode_test"])
        .assert()
        .code(0);
    reg_rs()
        .args(["report", "-p", "exitcode_test"])
        .assert()
        .code(0);

    // Set up .rgt test with mismatched baseline → exit 1
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(&lock_file);
    fs::write(&test_rgt, "command = \"echo changed\"\n").expect("failed to write test file");
    fs::write(&test_out, "original\n").expect("failed to write test file");

    reg_rs()
        .args(["run", "-p", "exitcode_test"])
        .assert()
        .code(1);
    reg_rs()
        .args(["report", "-p", "exitcode_test"])
        .assert()
        .code(1);

    // Error case — a data dir that cannot be created (beneath a file) → exit 2
    let not_a_dir = common::test_data_dir().join("exitcode_not_a_dir");
    fs::write(&not_a_dir, "").expect("failed to write test file");
    reg_rs()
        .args(["run", "-p", "foo"])
        .env("REG_RS_DATA_DIR", not_a_dir.join("data"))
        .assert()
        .code(2);

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(&lock_file);
    let _ = fs::remove_file(&test_rgt);
    let _ = fs::remove_file(&test_out);
    let _ = fs::remove_file(&not_a_dir);
}

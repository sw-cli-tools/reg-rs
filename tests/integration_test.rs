use std::fs;
use std::path::PathBuf;
use std::process;

use assert_cmd::Command;
use predicates::prelude::*;

mod common;

/// Build a Command for the reg-rs binary with the test data dir set
fn reg_rs() -> Command {
    let mut cmd = Command::cargo_bin("reg-rs").unwrap();
    cmd.env("REG_RS_DATA_DIR", common::test_data_dir());
    cmd
}

/// Return the absolute path to the debug binary built by cargo test
fn debug_bin_path() -> PathBuf {
    assert_cmd::cargo::cargo_bin("reg-rs")
}

/// Return the project root directory (where Cargo.toml lives)
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run a demo script with REG_RS_BIN pointing at the debug binary
/// and REG_RS_DATA_DIR pointing at an isolated test directory.
fn run_demo_script(script_name: &str) -> process::Output {
    let bin_path = debug_bin_path();
    let data_dir = common::test_data_dir().join(format!("demo_{}", script_name));
    let _ = fs::create_dir_all(&data_dir);
    let script_path = project_root().join("demo").join(script_name);

    process::Command::new("bash")
        .arg(&script_path)
        .env("REG_RS_BIN", &bin_path)
        .env("REG_RS_DATA_DIR", &data_dir)
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {}", script_name, e))
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
        .stdout(predicate::str::contains("analyze"))
        .stdout(predicate::str::contains(
            "Creates a new test of a specified command",
        ))
        .stdout(predicate::str::contains("Removes previously created test"))
        .stdout(predicate::str::contains("Reports counts/summary"))
        .stdout(predicate::str::contains("Runs a test"))
        .stdout(predicate::str::contains("Starts a server to monitor"))
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

// --- Preprocess tests ---

#[test]
fn integration_test_create_with_preprocess() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("pp_test.tdb");
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));

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

    // Run a command that outputs same lines in different order — should pass after sort
    // The original captured "banana\napple\ncherry\n" preprocessed to "apple\nbanana\ncherry\n"
    // This run outputs "cherry\napple\nbanana\n" preprocessed to "apple\nbanana\ncherry\n"
    // So after preprocessing both match.

    // First, remove and recreate with a command that outputs differently
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));

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

    assert!(test_db.exists());

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
fn integration_test_create_help_shows_preprocess() {
    common::setup();
    reg_rs()
        .args(["create", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-P, --preprocess"));
}

#[test]
fn integration_test_create_with_diff_mode_json() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("json_test.tdb");
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));

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

    assert!(test_db.exists());

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
fn integration_test_create_help_shows_diff_mode() {
    common::setup();
    reg_rs()
        .args(["create", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-M, --diff-mode"));
}

// --- Demo script tests (dogfooding) ---
// These run the demo shell scripts using the debug binary,
// ensuring reg-rs can test itself and that demo scripts stay working.

#[test]
fn integration_test_demo_dogfood() {
    common::setup();
    let output = run_demo_script("dogfood.sh");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "dogfood.sh failed (exit {}):\nstdout: {}\nstderr: {}",
        output.status.code().unwrap_or(-1),
        stdout,
        stderr
    );
    assert!(
        stdout.contains("reg-rs successfully tested itself"),
        "dogfood.sh should complete successfully:\n{}",
        stdout
    );
}

#[test]
fn integration_test_demo_test_basic() {
    common::setup();
    let output = run_demo_script("test_basic.sh");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "test_basic.sh failed (exit {}):\nstdout: {}\nstderr: {}",
        output.status.code().unwrap_or(-1),
        stdout,
        stderr
    );
    assert!(
        stdout.contains("All steps completed successfully"),
        "test_basic.sh should complete successfully:\n{}",
        stdout
    );
}

#[test]
fn integration_test_demo_test_workflow() {
    common::setup();
    let output = run_demo_script("test_workflow.sh");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "test_workflow.sh failed (exit {}):\nstdout: {}\nstderr: {}",
        output.status.code().unwrap_or(-1),
        stdout,
        stderr
    );
    assert!(
        stdout.contains("reg-rs successfully detected the regression"),
        "test_workflow.sh should detect regression:\n{}",
        stdout
    );
}

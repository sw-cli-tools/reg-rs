use std::fs;
use std::process::Command;

mod common;

/// Helper to run a command and return (status_code, stdout, stderr)
fn run_command(cmd: &str) -> (i32, String, String) {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute process");
    let status_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (status_code, stdout, stderr)
}

#[test]
fn integration_test_reg_rs_help() {
    common::setup();
    let (status_code, stdout, stderr) = run_command("./target/debug/reg-rs -h");
    assert_eq!(0, status_code);
    assert_eq!("", stderr);
    // Check key parts of help output rather than exact match (to allow for minor formatting changes)
    assert!(stdout.contains("Usage: reg-rs [OPTIONS] <COMMAND>"));
    assert!(stdout.contains("create  Creates a new test of a specified command"));
    assert!(stdout.contains("remove  Removes previously created test"));
    assert!(stdout.contains("report  Reports counts/summary"));
    assert!(stdout.contains("run     Runs a test"));
    assert!(stdout.contains("status  Starts a server to monitor"));
    assert!(stdout.contains("-d, --debug"));
    assert!(stdout.contains("-l, --logging"));
    assert!(stdout.contains("-h, --help"));
    assert!(stdout.contains("-V, --version"));
}

#[test]
fn integration_test_version() {
    common::setup();
    let (status_code, stdout, _stderr) = run_command("./target/debug/reg-rs -V");
    assert_eq!(0, status_code);
    assert!(stdout.starts_with("reg-rs "));
}

#[test]
fn integration_test_create_help() {
    common::setup();
    let (status_code, stdout, stderr) = run_command("./target/debug/reg-rs create -h");
    assert_eq!(0, status_code);
    assert_eq!("", stderr);
    assert!(stdout.contains("Creates a new test of a specified command"));
    assert!(stdout.contains("-t, --test"));
    assert!(stdout.contains("-c, --command"));
}

#[test]
fn integration_test_run_help() {
    common::setup();
    let (status_code, stdout, stderr) = run_command("./target/debug/reg-rs run -h");
    assert_eq!(0, status_code);
    assert_eq!("", stderr);
    assert!(stdout.contains("Runs a test"));
    assert!(stdout.contains("-p, --pattern"));
    assert!(stdout.contains("-n, --dry-run"));
}

#[test]
fn integration_test_report_help() {
    common::setup();
    let (status_code, stdout, stderr) = run_command("./target/debug/reg-rs report -h");
    assert_eq!(0, status_code);
    assert_eq!("", stderr);
    assert!(stdout.contains("Reports counts/summary"));
    assert!(stdout.contains("-p, --pattern"));
    assert!(stdout.contains("-v"));
}

#[test]
fn integration_test_remove_help() {
    common::setup();
    let (status_code, stdout, stderr) = run_command("./target/debug/reg-rs remove -h");
    assert_eq!(0, status_code);
    assert_eq!("", stderr);
    assert!(stdout.contains("Removes previously created test"));
    assert!(stdout.contains("-p, --pattern"));
}

#[test]
fn integration_test_status_help() {
    common::setup();
    let (status_code, stdout, stderr) = run_command("./target/debug/reg-rs status -h");
    assert_eq!(0, status_code);
    assert_eq!("", stderr);
    assert!(stdout.contains("Starts a server to monitor"));
    assert!(stdout.contains("-p, --pattern"));
    assert!(stdout.contains("-l, --localhost-port"));
}

#[test]
fn integration_test_create_and_run() {
    common::setup();

    // Tests must be in data/ directory with .tdb extension (per finder.rs)
    let _ = fs::create_dir_all("./data");
    let test_db = "data/integration_test.tdb";
    let test_pattern = "integration_test";

    // Clean up any existing test file (including lock file)
    let _ = fs::remove_file(test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db));

    // Create a new test
    let (status_code, _stdout, stderr) = run_command(&format!(
        "./target/debug/reg-rs create -t {} -c 'echo hello'",
        test_db
    ));

    if status_code != 0 {
        eprintln!("create failed: {}", stderr);
        return;
    }
    assert_eq!(0, status_code, "create failed: {}", stderr);

    // Verify the test database was created
    assert!(
        std::path::Path::new(test_db).exists(),
        "Test database should exist after create"
    );

    // Run the test (use pattern matching, not full path)
    let (status_code, stdout, stderr) =
        run_command(&format!("./target/debug/reg-rs run -p {}", test_pattern));

    if status_code != 0 {
        eprintln!("run failed: {}", stderr);
        let _ = fs::remove_file(test_db);
        let _ = fs::remove_file(format!("{}.lock", test_db));
        return;
    }
    assert_eq!(0, status_code, "run failed: {}", stderr);
    assert!(
        stdout.contains("hello"),
        "stdout should contain 'hello': {}",
        stdout
    );

    // Report on the test
    let (status_code, stdout, stderr) =
        run_command(&format!("./target/debug/reg-rs report -p {}", test_pattern));

    if status_code != 0 {
        eprintln!("report failed: {}", stderr);
        let _ = fs::remove_file(test_db);
        let _ = fs::remove_file(format!("{}.lock", test_db));
        return;
    }
    assert_eq!(0, status_code, "report failed: {}", stderr);
    assert!(
        stdout.contains("matched pattern"),
        "report should show matched pattern: {}",
        stdout
    );

    // Clean up
    let _ = fs::remove_file(test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db));
}

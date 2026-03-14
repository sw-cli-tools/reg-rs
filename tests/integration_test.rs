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
        .stdout(predicate::str::contains("Lists tests matching"))
        .stdout(predicate::str::contains("Shows detailed information"))
        .stdout(predicate::str::contains("Converts .tdb tests to .rgt"))
        .stdout(predicate::str::contains("Accepts latest test output"))
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
fn integration_test_create_help_shows_context() {
    common::setup();
    reg_rs()
        .args(["create", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-C, --context"));
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

#[test]
fn integration_test_create_with_diff_mode_lines_unordered() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("lines_test.tdb");
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));

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

    assert!(test_db.exists());

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

#[test]
fn integration_test_create_with_doc_metadata() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("doc_meta_test.tdb");
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));

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

    assert!(test_db.exists());

    // Clean up
    reg_rs()
        .args(["remove", "-p", "doc_meta_test"])
        .assert()
        .success();
}

// --- Status server and SSE tests ---

/// Start the status server on a given port and return the child process
fn start_status_server(port: u16, pattern: &str, data_dir: &std::path::Path) -> process::Child {
    let bin_path = debug_bin_path();
    process::Command::new(&bin_path)
        .args(["status", "-p", pattern, "-l", &port.to_string()])
        .env("REG_RS_DATA_DIR", data_dir)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .expect("failed to start status server")
}

/// Wait for the server to be ready by polling the given URL
fn wait_for_server(url: &str, timeout_secs: u64) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_secs(timeout_secs) {
        if ureq::get(url).call().is_ok() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    false
}

#[test]
fn integration_test_status_server_landing_page() {
    common::setup();

    let data_dir = common::test_data_dir().join("status_landing");
    let _ = fs::create_dir_all(&data_dir);
    let port = 14741_u16;

    // Create a test so the server has something to show
    let bin_path = debug_bin_path();
    let _ = process::Command::new(&bin_path)
        .args(["create", "-t", "status_test", "-c", "echo hi"])
        .env("REG_RS_DATA_DIR", &data_dir)
        .output();

    let mut server = start_status_server(port, "status_test", &data_dir);
    let base_url = format!("http://127.0.0.1:{}", port);

    assert!(
        wait_for_server(&base_url, 10),
        "Server did not start within 10s"
    );

    // Check landing page
    let resp = ureq::get(&base_url)
        .call()
        .expect("landing page GET failed");
    assert_eq!(resp.status(), 200);
    let body = resp.into_string().unwrap();
    assert!(body.contains("reg-rs"), "Landing page should contain title");
    assert!(
        body.contains("status_test"),
        "Landing page should show pattern"
    );
    assert!(
        body.contains("EventSource"),
        "Landing page should have SSE client"
    );
    assert!(
        body.contains("sse-count"),
        "Landing page should have SSE counter"
    );

    // Check status dashboard
    let resp = ureq::get(&format!("{}/status", base_url))
        .call()
        .expect("status page GET failed");
    assert_eq!(resp.status(), 200);
    let body = resp.into_string().unwrap();
    assert!(body.contains("status_test"), "Status page should show test");

    // Check API endpoint
    let resp = ureq::get(&format!("{}/api/status", base_url))
        .call()
        .expect("api/status GET failed");
    assert_eq!(resp.status(), 200);
    let json: serde_json::Value = resp.into_json().unwrap();
    assert_eq!(json["pattern"], "status_test");
    assert!(json["total_count"].as_u64().unwrap() >= 1);

    server.kill().expect("failed to kill server");
    let _ = server.wait();

    // Clean up
    let _ = fs::remove_file(data_dir.join("status_test.tdb"));
    let _ = fs::remove_file(data_dir.join("status_test.tdb.lock"));
}

#[test]
fn integration_test_sse_broadcasts_on_file_change() {
    common::setup();

    let data_dir = common::test_data_dir().join("sse_broadcast");
    let _ = fs::create_dir_all(&data_dir);
    let port = 14742_u16;

    // Create a test so the server watches the directory
    let bin_path = debug_bin_path();
    let _ = process::Command::new(&bin_path)
        .args(["create", "-t", "sse_test", "-c", "echo hi"])
        .env("REG_RS_DATA_DIR", &data_dir)
        .output();

    let mut server = start_status_server(port, "sse_test", &data_dir);
    let base_url = format!("http://127.0.0.1:{}", port);

    assert!(
        wait_for_server(&base_url, 10),
        "Server did not start within 10s"
    );

    // Connect to SSE endpoint — use a thread with a timeout
    // Get initial state via API
    let resp = ureq::get(&format!("{}/api/status", base_url))
        .call()
        .expect("api GET failed");
    let initial: serde_json::Value = resp.into_json().unwrap();
    let initial_updated = initial["updated"].as_str().unwrap_or("").to_string();

    // Trigger a file change in the data directory (run the test)
    let _ = process::Command::new(&bin_path)
        .args(["run", "-p", "sse_test"])
        .env("REG_RS_DATA_DIR", &data_dir)
        .output();

    // Wait for the file watcher debounce + processing
    std::thread::sleep(std::time::Duration::from_secs(4));

    // Check that the state was updated
    let resp = ureq::get(&format!("{}/api/status", base_url))
        .call()
        .expect("api GET failed after change");
    let updated: serde_json::Value = resp.into_json().unwrap();
    let new_updated = updated["updated"].as_str().unwrap_or("").to_string();

    // The updated timestamp should have changed
    assert_ne!(
        initial_updated, new_updated,
        "State should update after file change (initial: {}, new: {})",
        initial_updated, new_updated
    );

    // Verify the test now shows as having been run
    let tests = updated["tests"]
        .as_array()
        .unwrap_or_else(|| panic!("expected tests array in response: {}", updated));
    let sse_test = tests
        .iter()
        .find(|t| t["name"].as_str().unwrap_or("").contains("sse_test"))
        .unwrap_or_else(|| panic!("expected sse_test in tests: {:?}", tests));
    assert!(
        sse_test["last_ran"].is_string(),
        "Test should have a last_ran timestamp after running"
    );

    server.kill().expect("failed to kill server");
    let _ = server.wait();

    // Clean up
    let _ = fs::remove_file(data_dir.join("sse_test.tdb"));
    let _ = fs::remove_file(data_dir.join("sse_test.tdb.lock"));
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
fn integration_test_migrate_tdb_to_rgt() {
    common::setup();

    let data_dir = common::test_data_dir();
    let test_db = data_dir.join("migrate_test.tdb");
    let test_rgt = data_dir.join("migrate_test.rgt");
    let test_out = data_dir.join("migrate_test.out");

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));
    let _ = fs::remove_file(&test_rgt);
    let _ = fs::remove_file(&test_out);

    // Create a .tdb test with metadata
    reg_rs()
        .args([
            "create",
            "-t",
            "migrate_test",
            "-c",
            "echo migrated",
            "--desc",
            "A migration test",
        ])
        .assert()
        .success();

    assert!(test_db.exists());
    assert!(!test_rgt.exists());

    // Migrate
    reg_rs()
        .args(["migrate", "-p", "migrate_test"])
        .assert()
        .success()
        .stderr(predicate::str::contains("migrated:"))
        .stderr(predicate::str::contains("1 test(s) migrated"));

    // .rgt and .out should now exist
    assert!(test_rgt.exists(), ".rgt file should exist after migrate");
    assert!(test_out.exists(), ".out file should exist after migrate");

    // .rgt should contain the command
    let rgt_content = fs::read_to_string(&test_rgt).unwrap();
    assert!(
        rgt_content.contains("echo migrated"),
        ".rgt should contain command"
    );
    assert!(
        rgt_content.contains("A migration test"),
        ".rgt should contain desc"
    );

    // .out should contain the baseline stdout
    let out_content = fs::read_to_string(&test_out).unwrap();
    assert_eq!(out_content, "migrated\n");

    // Clean up
    let _ = fs::remove_file(&test_db);
    let _ = fs::remove_file(format!("{}.lock", test_db.display()));
    let _ = fs::remove_file(&test_rgt);
    let _ = fs::remove_file(&test_out);
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
    fs::write(&test_rgt, "command = \"echo rebased\"\n").unwrap();
    fs::write(&test_out, "original output\n").unwrap();

    // Run the .rgt test — will produce "rebased\n" which differs from "original output\n"
    reg_rs()
        .args(["run", "-p", "rebase_test"])
        .assert()
        .success();

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
    let out_content = fs::read_to_string(&test_out).unwrap();
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

//! The status web server: landing page and SSE updates.

use std::fs;
use std::process;

mod common;
use common::debug_bin_path;

use std::sync::atomic::{AtomicU16, Ordering};

/// Atomic port counter to ensure each test gets a unique port.
/// Starts high to avoid conflicts with other services.
static NEXT_PORT: AtomicU16 = AtomicU16::new(24700);

/// Allocate a unique port for a test server.
fn allocate_port() -> u16 {
    NEXT_PORT.fetch_add(1, Ordering::SeqCst)
}

/// Kill any process listening on the given port (belt).
fn kill_port_holder(port: u16) {
    let _ = process::Command::new("lsof")
        .args(["-ti", &format!(":{port}")])
        .output()
        .ok()
        .map(|out| {
            let pids = String::from_utf8_lossy(&out.stdout);
            for pid in pids.split_whitespace() {
                let _ = process::Command::new("kill").arg(pid).output();
            }
        });
    std::thread::sleep(std::time::Duration::from_millis(100));
}

/// RAII guard that kills the server child on drop.
struct ServerGuard {
    child: process::Child,
    port: u16,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        kill_port_holder(self.port);
    }
}

/// Start the status server with robust port handling.
///
/// 1. Kills any stale process on the port (belt)
/// 2. Spawns the server subprocess
/// 3. Verifies the server is reachable (suspenders)
fn start_status_server_robust(pattern: &str, data_dir: &std::path::Path) -> (ServerGuard, String) {
    let port = allocate_port();
    kill_port_holder(port);

    let bin_path = debug_bin_path();
    let child = process::Command::new(&bin_path)
        .args(["status", "-p", pattern, "-l", &port.to_string()])
        .env("REG_RS_DATA_DIR", data_dir)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .expect("failed to start status server");

    let base_url = format!("http://127.0.0.1:{port}");

    // Verify the server bound successfully by polling
    assert!(
        wait_for_server(&base_url, 10),
        "Server on port {port} did not start within 10s"
    );

    (ServerGuard { child, port }, base_url)
}

/// Wait for the API to report at least one test, retrying up to `timeout_secs`.
fn wait_for_api_tests(base_url: &str, timeout_secs: u64) -> serde_json::Value {
    let api_url = format!("{base_url}/api/status");
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_secs(timeout_secs) {
        if let Ok(resp) = ureq::get(&api_url).call()
            && let Ok(json) = resp.into_json::<serde_json::Value>()
            && json["total_count"].as_u64().unwrap_or(0) > 0
        {
            return json;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    panic!("API did not report tests within {timeout_secs}s");
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
    let _ = fs::remove_dir_all(&data_dir);
    let _ = fs::create_dir_all(&data_dir);

    // Create a test so the server has something to show
    let bin_path = debug_bin_path();
    let create_out = process::Command::new(&bin_path)
        .args(["create", "-t", "status_test", "-c", "echo hi"])
        .env("REG_RS_DATA_DIR", &data_dir)
        .output()
        .expect("failed to run create");
    assert!(create_out.status.success(), "create failed");

    let (_guard, base_url) = start_status_server_robust("status_test", &data_dir);

    // Check landing page
    let resp = ureq::get(&base_url)
        .call()
        .expect("landing page GET failed");
    assert_eq!(resp.status(), 200);
    let body = resp.into_string().expect("failed to read response body");
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
        body.contains("sse-badge"),
        "Landing page should have SSE counter badge"
    );

    // Check status dashboard
    let resp = ureq::get(&format!("{base_url}/status"))
        .call()
        .expect("status page GET failed");
    assert_eq!(resp.status(), 200);
    let body = resp.into_string().expect("failed to read response body");
    assert!(body.contains("status_test"), "Status page should show test");

    // Check API endpoint
    let resp = ureq::get(&format!("{base_url}/api/status"))
        .call()
        .expect("api/status GET failed");
    assert_eq!(resp.status(), 200);
    let json: serde_json::Value = resp.into_json().expect("failed to parse JSON response");
    assert_eq!(json["pattern"], "status_test");
    assert!(
        json["total_count"]
            .as_u64()
            .expect("expected u64 field in JSON")
            >= 1
    );
    // _guard drops here, killing the server and freeing the port
}

#[test]
fn integration_test_sse_broadcasts_on_file_change() {
    common::setup();

    let data_dir = common::test_data_dir().join("sse_broadcast");
    // Clean slate to prevent stale data from prior runs
    let _ = fs::remove_dir_all(&data_dir);
    let _ = fs::create_dir_all(&data_dir);

    // Create a test so the server watches the directory
    let bin_path = debug_bin_path();
    let create_output = process::Command::new(&bin_path)
        .args(["create", "-t", "sse_test", "-c", "echo hi"])
        .env("REG_RS_DATA_DIR", &data_dir)
        .output()
        .expect("failed to run create");
    assert!(
        create_output.status.success(),
        "create failed: {}",
        String::from_utf8_lossy(&create_output.stderr)
    );
    assert!(
        data_dir.join("sse_test.rgt").exists(),
        "sse_test.rgt should exist after create"
    );

    let (_guard, base_url) = start_status_server_robust("sse_test", &data_dir);

    // Wait for server to populate initial test state (retry until tests appear)
    let initial = wait_for_api_tests(&base_url, 10);
    let initial_updated = initial["updated"].as_str().unwrap_or("").to_string();

    // Trigger a file change in the data directory (run the test)
    let _ = process::Command::new(&bin_path)
        .args(["run", "-p", "sse_test"])
        .env("REG_RS_DATA_DIR", &data_dir)
        .output();

    // Wait for the file watcher debounce (2s) + processing
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Check that the state was updated
    let resp = ureq::get(&format!("{base_url}/api/status"))
        .call()
        .expect("api GET failed after change");
    let updated: serde_json::Value = resp.into_json().expect("failed to parse JSON response");
    let new_updated = updated["updated"].as_str().unwrap_or("").to_string();

    // The updated timestamp should have changed
    assert_ne!(
        initial_updated, new_updated,
        "State should update after file change (initial: {initial_updated}, new: {new_updated})"
    );

    // Verify the test now shows as having been run
    let tests = updated["tests"]
        .as_array()
        .unwrap_or_else(|| panic!("expected tests array in response: {updated}"));
    let sse_test = tests
        .iter()
        .find(|t| t["name"].as_str().unwrap_or("").contains("sse_test"))
        .unwrap_or_else(|| panic!("expected sse_test in tests: {tests:?}"));
    assert!(
        sse_test["last_ran"].is_string(),
        "Test should have a last_ran timestamp after running"
    );
    // _guard drops here, killing the server and freeing the port

    // Clean up
    let _ = fs::remove_file(data_dir.join("sse_test.tdb"));
    let _ = fs::remove_file(data_dir.join("sse_test.tdb.lock"));
}

// --- Demo script tests (dogfooding) ---
// These run the demo shell scripts using the debug binary,
// ensuring reg-rs can test itself and that demo scripts stay working.

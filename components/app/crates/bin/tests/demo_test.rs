//! The demo scripts (demo/*.sh), run against the debug binary.

use std::fs;
use std::process;

mod common;
use common::{debug_bin_path, project_root};

/// A `bash` command. On Windows, `bash` on `PATH` can be System32's WSL
/// launcher, so go through Git Bash's `sh` (the one reg-rs itself uses).
fn bash() -> process::Command {
    if cfg!(windows) {
        let mut cmd = process::Command::new("sh");
        cmd.args(["-c", r#"exec bash "$0" "$@""#]);
        cmd
    } else {
        process::Command::new("bash")
    }
}

/// A path with forward slashes, which both Windows and `sh` accept
/// (backslashes are escapes inside the shell commands the demos build).
fn sh_path(path: &std::path::Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// Run a demo script with REG_RS_BIN pointing at the debug binary
/// and REG_RS_DATA_DIR pointing at an isolated test directory.
fn run_demo_script(script_name: &str) -> process::Output {
    let bin_path = debug_bin_path();
    let data_dir = common::test_data_dir().join(format!("demo_{script_name}"));
    let _ = fs::create_dir_all(&data_dir);
    let script_path = project_root().join("demo").join(script_name);

    bash()
        .arg(sh_path(&script_path))
        .env("REG_RS_BIN", sh_path(&bin_path))
        .env("REG_RS_DATA_DIR", sh_path(&data_dir))
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to run {script_name}: {e}"))
}

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
        "dogfood.sh should complete successfully:\n{stdout}"
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
        "test_basic.sh should complete successfully:\n{stdout}"
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
        "test_workflow.sh should detect regression:\n{stdout}"
    );
}

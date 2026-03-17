use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set by cargo");
    let dest_path = Path::new(&out_dir).join("generated.rs");

    let version = env!("CARGO_PKG_VERSION");
    let build_time = chrono::Local::now().to_rfc3339();
    let build_host = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "unknown".to_string());

    let commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let long_about = "reg-rs (regress) - A CLI for regression testing

reg-rs captures command output and exit codes as baseline 'golden' results,
then compares subsequent runs against these baselines to detect regressions.

Tests are stored in ~/.local/reg-rs/ by default (override with REG_RS_DATA_DIR).

WORKFLOW:
  1. Create a test:   reg-rs create -t my_test -c 'my_command'
  2. Run the test:    reg-rs run -p my_test
  3. View results:    reg-rs report -p my_test -v

AI CODING AGENT INSTRUCTIONS:
  This tool is designed to be used by AI coding agents to ensure system integrity.
  1. Use 'reg-rs create' to establish baselines for new features.
  2. Use 'reg-rs run' to verify that changes haven't introduced regressions.
  3. Use 'reg-rs status' for real-time monitoring of long-running test suites.
  4. Always address failures before committing code.";

    let content = format!(
        r#"
/// Version information (short and long are identical for sw-checklist compliance)
pub fn generated_version() -> &'static str {{
    "{version}
Copyright (c) 2026 Mike Wright
License: MIT
Repository: https://github.com/softwarewrighter/sw-cli-tools
Build Host: {build_host}
Build Commit: {commit}
Build Time: {build_time}"
}}

/// Extended version information for --version
pub fn extended_version() -> &'static str {{
    generated_version()
}}

/// Long about text for help
pub fn generated_long_about() -> &'static str {{
    "{long_about}"
}}
"#,
        version = version,
        build_host = build_host,
        commit = commit,
        build_time = build_time,
        long_about = long_about.replace('"', "\\\"")
    );

    fs::write(&dest_path, content).expect("failed to write generated.rs");
}

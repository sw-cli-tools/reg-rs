use std::time::Duration;

use reg_rs_exec::executor::CommandExecutor;
use reg_rs_exec::process;
use reg_rs_types::types::TestResults;

/// Current time as formatted string
fn now() -> String {
    let date = chrono::Local::now();
    format!("{}", date.format("%Y-%m-%dT%H:%M:%S"))
}

/// Run one test with a custom timeout (in seconds)
pub fn run_one_timeout(
    test_name: &str,
    command: &str,
    dry_run: bool,
    timeout_secs: u64,
) -> reg_rs_types::error::Result<Option<TestResults>> {
    log::info!(
        "runner/run_one_timeout test_name {}, dry_run {}, timeout {}s",
        &test_name,
        dry_run,
        timeout_secs
    );
    if dry_run {
        println!("dry-run: test name: {test_name}, command: {command}");
        Ok(None)
    } else {
        let (exit_code, stderr, stdout) =
            process::exec_with_timeout(command.to_string(), Duration::from_secs(timeout_secs))?;
        let test = TestResults {
            name: test_name.to_string(),
            command: command.to_string(),
            time_created: now(),
            exit_code,
            stderr,
            stdout,
        };
        Ok(Some(test))
    }
}

/// Run one test with default timeout
pub fn run_one(
    test_name: &str,
    command: &str,
    dry_run: bool,
) -> reg_rs_types::error::Result<Option<TestResults>> {
    log::info!(
        "runner/run_one test_name {}, dry_run {}",
        &test_name,
        dry_run
    );
    if dry_run {
        println!("dry-run: test name: {test_name}, command: {command}");
        Ok(None)
    } else {
        let (exit_code, stderr, stdout) = process::exec(command.to_string())?;
        let test = TestResults {
            name: test_name.to_string(),
            command: command.to_string(),
            time_created: now(),
            exit_code,
            stderr,
            stdout,
        };
        Ok(Some(test))
    }
}

/// Run one test with a custom executor (for dependency injection)
pub fn run_one_with_executor(
    test_name: &str,
    command: &str,
    dry_run: bool,
    executor: &dyn CommandExecutor,
) -> reg_rs_types::error::Result<Option<TestResults>> {
    log::info!(
        "runner/run_one_with_executor test_name {}, dry_run {}",
        &test_name,
        dry_run
    );
    if dry_run {
        println!("dry-run: test name: {test_name}, command: {command}");
        Ok(None)
    } else {
        let (exit_code, stderr, stdout) = executor.exec(command)?;
        let test = TestResults {
            name: test_name.to_string(),
            command: command.to_string(),
            time_created: now(),
            exit_code,
            stderr,
            stdout,
        };
        Ok(Some(test))
    }
}

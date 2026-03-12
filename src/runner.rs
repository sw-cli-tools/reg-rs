use std::sync::Mutex;
use std::time::Duration;

use crate::config;
use crate::db;
use crate::diff;
use crate::executor::CommandExecutor;
use crate::finder;
use crate::process;
use crate::time;

/// Test Results data
#[derive(Debug)]
pub struct TestResults {
    /// Test Name
    pub name: String,
    /// Subject command
    pub command: String,
    /// test results creation time
    pub time_created: String,
    /// test exit code
    pub exit_code: i32,
    /// test captured stderr
    pub stderr: String,
    /// test captured stdout
    pub stdout: String,
}

/// Run many tests (sequential or parallel based on config)
pub fn run_many(config: &config::Config) -> crate::error::Result<()> {
    log::info!("runner/run_many");
    let pattern = config.extract_pattern().to_string();
    let result = finder::discover(pattern.clone())?;
    if config.debug {
        log::debug!("run_many config: {:?}", &config);
        log::debug!("run_many tests: {:?}", &result);
    }
    if result.found.is_empty() {
        eprintln!(
            "warning: no tests matched pattern '{}' in {}",
            pattern,
            result.data_dir.display()
        );
        return Ok(());
    }
    let tests = result;
    if config.is_parallel() {
        run_many_parallel(&tests.found, config.is_dry_run())
    } else {
        run_many_sequential(&tests.found, config.is_dry_run())
    }
}

/// Run tests sequentially
fn run_many_sequential(tests: &[String], dry_run: bool) -> crate::error::Result<()> {
    for test in tests {
        run_and_diff(test, dry_run)?;
    }
    Ok(())
}

/// Run tests in parallel using scoped threads (one thread per test)
fn run_many_parallel(tests: &[String], dry_run: bool) -> crate::error::Result<()> {
    let test_count = tests.len();
    eprintln!("running {} tests in parallel", test_count);
    let start = std::time::Instant::now();

    let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());

    std::thread::scope(|s| {
        for test in tests {
            let errors = &errors;
            s.spawn(move || {
                if let Err(e) = run_and_diff(test, dry_run) {
                    errors.lock().unwrap().push(format!("{}: {}", test, e));
                }
            });
        }
    });

    let elapsed = start.elapsed();
    eprintln!(
        "parallel run complete: {} tests in {:.2}s",
        test_count,
        elapsed.as_secs_f64()
    );

    let errors = errors.into_inner().unwrap();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(crate::error::RegError::Other(format!(
            "{} test(s) failed:\n  {}",
            errors.len(),
            errors.join("\n  ")
        )))
    }
}

/// Run a single test and process differences
fn run_and_diff(test: &str, dry_run: bool) -> crate::error::Result<()> {
    let prior_test_result = db::read_original_results(test)?;
    let timeout_secs = db::read_metadata(test, "timeout")?
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(300);
    let maybe_regression =
        run_one_timeout(test, &prior_test_result.command, dry_run, timeout_secs)?;
    if let Some(latest_test_result) = maybe_regression {
        diff::process_differences(test, &prior_test_result, &latest_test_result)?;
        db::replace_latest_results(test, &latest_test_result)?;
    }
    Ok(())
}

/// Run one test with a custom timeout (in seconds)
pub fn run_one_timeout(
    test_name: &str,
    command: &str,
    dry_run: bool,
    timeout_secs: u64,
) -> crate::error::Result<Option<TestResults>> {
    log::info!(
        "runner/run_one_timeout test_name {}, dry_run {}, timeout {}s",
        &test_name,
        dry_run,
        timeout_secs
    );
    if dry_run {
        println!("dry-run: test name: {}, command: {}", test_name, command);
        Ok(None)
    } else {
        let (exit_code, stderr, stdout) =
            process::exec_with_timeout(command.to_string(), Duration::from_secs(timeout_secs))?;
        let test = TestResults {
            name: test_name.to_string(),
            command: command.to_string(),
            time_created: time::now(),
            exit_code,
            stderr,
            stdout,
        };
        Ok(Some(test))
    }
}

/// Run one test
pub fn run_one(
    test_name: &str,
    command: &str,
    dry_run: bool,
) -> crate::error::Result<Option<TestResults>> {
    log::info!(
        "runner/run_one test_name {}, dry_run {}",
        &test_name,
        dry_run
    );
    if dry_run {
        println!("dry-run: test name: {}, command: {}", test_name, command);
        Ok(None)
    } else {
        let (exit_code, stderr, stdout) = process::exec(command.to_string())?;
        let test = TestResults {
            name: test_name.to_string(),
            command: command.to_string(),
            time_created: time::now(),
            exit_code,
            stderr,
            stdout,
        };
        // db write regression results (maybe_create, update)
        Ok(Some(test))
    }
}

/// Run one test with a custom executor (for dependency injection)
///
/// This function allows injecting a custom command executor,
/// which is useful for testing without actually running commands.
pub fn run_one_with_executor(
    test_name: &str,
    command: &str,
    dry_run: bool,
    executor: &dyn CommandExecutor,
) -> crate::error::Result<Option<TestResults>> {
    log::info!(
        "runner/run_one_with_executor test_name {}, dry_run {}",
        &test_name,
        dry_run
    );
    if dry_run {
        println!("dry-run: test name: {}, command: {}", test_name, command);
        Ok(None)
    } else {
        let (exit_code, stderr, stdout) = executor.exec(command)?;
        let test = TestResults {
            name: test_name.to_string(),
            command: command.to_string(),
            time_created: time::now(),
            exit_code,
            stderr,
            stdout,
        };
        Ok(Some(test))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::mock::MockCommandExecutor;

    #[test]
    fn test_run_one_with_executor_dry_run() {
        let executor = MockCommandExecutor::success("hello\n");
        let result = run_one_with_executor("test1", "echo hello", true, &executor).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_run_one_with_executor_success() {
        let executor = MockCommandExecutor::success("hello world\n");
        let result = run_one_with_executor("test1", "echo hello world", false, &executor).unwrap();
        assert!(result.is_some());
        let test_results = result.unwrap();
        assert_eq!(test_results.name, "test1");
        assert_eq!(test_results.command, "echo hello world");
        assert_eq!(test_results.exit_code, 0);
        assert_eq!(test_results.stdout, "hello world\n");
    }

    #[test]
    fn test_run_one_with_executor_failure() {
        let executor = MockCommandExecutor::failure(1, "error message");
        let result = run_one_with_executor("test1", "failing command", false, &executor).unwrap();
        assert!(result.is_some());
        let test_results = result.unwrap();
        assert_eq!(test_results.exit_code, 1);
        assert_eq!(test_results.stderr, "error message");
    }
}

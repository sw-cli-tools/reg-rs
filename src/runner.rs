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

/// Run many tests
pub fn run_many(config: &config::Config) -> crate::error::Result<()> {
    log::info!("runner/run_many");
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern.clone())?;
    if config.debug {
        log::debug!("run_many config: {:?}", &config);
        log::debug!("run_many tests: {:?}", &tests);
    }
    if tests.found.is_empty() {
        eprintln!("warning: no tests matched pattern '{}'", pattern);
        return Ok(());
    }
    for test in tests.found {
        let prior_test_result = db::read_original_results(&test)?;
        let maybe_regression = run_one(&test, &prior_test_result.command, config.is_dry_run())?;
        if let Some(latest_test_result) = maybe_regression {
            let db_name = &test;
            diff::process_differences(db_name, &prior_test_result, &latest_test_result)?;
            db::replace_latest_results(db_name, &latest_test_result)?;
        }
    }
    Ok(())
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

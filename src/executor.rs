//! Command executor trait and implementations for dependency injection

use crate::error::RttError;

/// Trait for executing shell commands
///
/// This trait enables dependency injection for testing by allowing
/// mock implementations to be substituted for the real command executor.
pub trait CommandExecutor: Send + Sync {
    /// Execute a shell command and return (exit_code, stderr, stdout)
    fn exec(&self, command: &str) -> Result<(i32, String, String), Box<dyn std::error::Error>>;
}

/// Real command executor that runs commands via the shell
#[derive(Debug, Default)]
pub struct RealCommandExecutor;

impl RealCommandExecutor {
    /// Create a new RealCommandExecutor
    pub fn new() -> Self {
        Self
    }
}

impl CommandExecutor for RealCommandExecutor {
    fn exec(&self, command: &str) -> Result<(i32, String, String), Box<dyn std::error::Error>> {
        use std::process::Command;

        log::info!("executor/exec command: {}", command);
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|e| {
                RttError::CommandExecution(format!("failed to execute '{}': {}", command, e))
            })?;
        let status_code = output.status.code().unwrap_or(-1);

        println!("status: {:#?} status_code:{}", output.status, status_code);
        println!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        println!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));

        let exit = status_code;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok((exit, stderr, stdout))
    }
}

/// Mock command executor for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::Mutex;

    /// Mock command executor that returns predefined responses
    pub struct MockCommandExecutor {
        /// Expected responses: (exit_code, stderr, stdout)
        responses: Mutex<Vec<(i32, String, String)>>,
    }

    impl MockCommandExecutor {
        /// Create a new MockCommandExecutor with predefined responses
        pub fn new(responses: Vec<(i32, String, String)>) -> Self {
            Self {
                responses: Mutex::new(responses),
            }
        }

        /// Create a MockCommandExecutor that always succeeds with given output
        pub fn success(stdout: &str) -> Self {
            Self::new(vec![(0, String::new(), stdout.to_string())])
        }

        /// Create a MockCommandExecutor that always fails
        pub fn failure(exit_code: i32, stderr: &str) -> Self {
            Self::new(vec![(exit_code, stderr.to_string(), String::new())])
        }
    }

    impl CommandExecutor for MockCommandExecutor {
        fn exec(
            &self,
            _command: &str,
        ) -> Result<(i32, String, String), Box<dyn std::error::Error>> {
            let mut responses = self.responses.lock().unwrap();
            if responses.is_empty() {
                Ok((0, String::new(), String::new()))
            } else {
                Ok(responses.remove(0))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_mock_executor_success() {
            let executor = MockCommandExecutor::success("hello world\n");
            let (exit, stderr, stdout) = executor.exec("echo hello world").unwrap();
            assert_eq!(exit, 0);
            assert_eq!(stderr, "");
            assert_eq!(stdout, "hello world\n");
        }

        #[test]
        fn test_mock_executor_failure() {
            let executor = MockCommandExecutor::failure(1, "command not found");
            let (exit, stderr, stdout) = executor.exec("nonexistent").unwrap();
            assert_eq!(exit, 1);
            assert_eq!(stderr, "command not found");
            assert_eq!(stdout, "");
        }

        #[test]
        fn test_mock_executor_multiple_responses() {
            let executor = MockCommandExecutor::new(vec![
                (0, String::new(), "first\n".to_string()),
                (0, String::new(), "second\n".to_string()),
            ]);

            let (_, _, stdout) = executor.exec("first").unwrap();
            assert_eq!(stdout, "first\n");

            let (_, _, stdout) = executor.exec("second").unwrap();
            assert_eq!(stdout, "second\n");
        }
    }
}

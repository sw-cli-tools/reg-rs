use reg_rs_exec::executor::CommandExecutor;
use reg_rs_types::error::Result;
use std::sync::Mutex;

/// Mock command executor that returns predefined responses
pub struct MockCommandExecutor {
    responses: Mutex<Vec<(i32, String, String)>>,
}

impl MockCommandExecutor {
    /// Create a mock executor with a list of responses
    pub fn new(responses: Vec<(i32, String, String)>) -> Self {
        Self {
            responses: Mutex::new(responses),
        }
    }

    /// Create a mock executor that returns a single success response
    pub fn success(stdout: &str) -> Self {
        Self::new(vec![(0, String::new(), stdout.to_string())])
    }

    /// Create a mock executor that returns a single failure response
    pub fn failure(exit_code: i32, stderr: &str) -> Self {
        Self::new(vec![(exit_code, stderr.to_string(), String::new())])
    }
}

impl CommandExecutor for MockCommandExecutor {
    fn exec(&self, _command: &str) -> Result<(i32, String, String)> {
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Ok((0, String::new(), String::new()))
        } else {
            Ok(responses.remove(0))
        }
    }
}

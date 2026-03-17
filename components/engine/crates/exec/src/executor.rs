use reg_rs_types::error::Result;

/// Trait for executing shell commands
///
/// This trait enables dependency injection for testing by allowing
/// mock implementations to be substituted for the real command executor.
pub trait CommandExecutor: Send + Sync {
    /// Execute a shell command and return (exit_code, stderr, stdout)
    fn exec(&self, command: &str) -> Result<(i32, String, String)>;
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
    fn exec(&self, command: &str) -> Result<(i32, String, String)> {
        crate::process::exec(command.to_string())
    }
}

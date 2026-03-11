use std::io;
use std::path::PathBuf;
use std::result;
use thiserror::Error;

/// Custom error type for reg-rs
#[derive(Error, Debug)]
pub enum RegError {
    /// File I/O error
    #[error("File I/O error: {0}")]
    Io(#[from] io::Error),

    /// SQLite database error
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// File locking error
    #[error("File lock error: {0}")]
    FileLock(String),

    /// Test not found error
    #[error("Test not found: {0}")]
    TestNotFound(String),

    /// Command execution error
    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    /// Path-related error
    #[error("Path error for {path}: {message}")]
    Path {
        /// The path that caused the error
        path: PathBuf,
        /// A descriptive error message
        message: String,
    },

    /// Notification system error
    #[error("Notification error: {0}")]
    Notification(String),

    /// Template rendering error
    #[error("Template error: {0}")]
    Template(String),

    /// Web server error
    #[error("Web server error: {0}")]
    WebServer(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Mutex poisoned error
    #[error("Mutex poisoned: {0}")]
    MutexPoisoned(String),

    /// Other general errors
    #[error("{0}")]
    Other(String),
}

/// Result type alias with RegError as the error type
pub type Result<T> = result::Result<T, RegError>;

// Conversion implementations for common error types
// io::Error From impl is auto-derived by thiserror

impl From<tinytemplate::error::Error> for RegError {
    fn from(err: tinytemplate::error::Error) -> Self {
        RegError::Template(err.to_string())
    }
}

impl From<notify::Error> for RegError {
    fn from(err: notify::Error) -> Self {
        RegError::Notification(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for RegError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        RegError::Other(err.to_string())
    }
}

impl From<String> for RegError {
    fn from(err: String) -> Self {
        RegError::Other(err)
    }
}

impl From<&str> for RegError {
    fn from(err: &str) -> Self {
        RegError::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_io_error() {
        let err = RegError::Io(io::Error::new(io::ErrorKind::NotFound, "missing"));
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn test_display_file_lock() {
        let err = RegError::FileLock("locked".to_string());
        assert_eq!(err.to_string(), "File lock error: locked");
    }

    #[test]
    fn test_display_test_not_found() {
        let err = RegError::TestNotFound("my_test".to_string());
        assert_eq!(err.to_string(), "Test not found: my_test");
    }

    #[test]
    fn test_display_command_execution() {
        let err = RegError::CommandExecution("timeout".to_string());
        assert_eq!(err.to_string(), "Command execution failed: timeout");
    }

    #[test]
    fn test_display_path_error() {
        let err = RegError::Path {
            path: PathBuf::from("/tmp/test"),
            message: "not found".to_string(),
        };
        assert!(err.to_string().contains("/tmp/test"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_display_template() {
        let err = RegError::Template("bad template".to_string());
        assert_eq!(err.to_string(), "Template error: bad template");
    }

    #[test]
    fn test_display_other() {
        let err = RegError::Other("something".to_string());
        assert_eq!(err.to_string(), "something");
    }

    #[test]
    fn test_from_string() {
        let err: RegError = "string error".to_string().into();
        assert_eq!(err.to_string(), "string error");
    }

    #[test]
    fn test_from_str() {
        let err: RegError = "str error".into();
        assert_eq!(err.to_string(), "str error");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let err: RegError = io_err.into();
        assert!(err.to_string().contains("denied"));
    }

    #[test]
    fn test_from_box_dyn_error() {
        let boxed: Box<dyn std::error::Error> = "boxed error".into();
        let err: RegError = boxed.into();
        assert_eq!(err.to_string(), "boxed error");
    }
}

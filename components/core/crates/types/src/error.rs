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

    /// SQLite database error (string-based to avoid rusqlite dependency in types)
    #[error("Database error: {0}")]
    Database(String),

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

impl From<Box<dyn std::error::Error>> for RegError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        RegError::Other(err.to_string())
    }
}

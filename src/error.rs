use std::io;
use std::path::PathBuf;
use std::result;
use thiserror::Error;

/// Custom error type for RTT1
#[derive(Error, Debug)]
pub enum RttError {
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

/// Result type alias with RttError as the error type
pub type Result<T> = result::Result<T, RttError>;

// Conversion implementations for common error types
// io::Error From impl is auto-derived by thiserror

impl From<tinytemplate::error::Error> for RttError {
    fn from(err: tinytemplate::error::Error) -> Self {
        RttError::Template(err.to_string())
    }
}

impl From<notify::Error> for RttError {
    fn from(err: notify::Error) -> Self {
        RttError::Notification(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for RttError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        RttError::Other(err.to_string())
    }
}

impl From<String> for RttError {
    fn from(err: String) -> Self {
        RttError::Other(err)
    }
}

impl From<&str> for RttError {
    fn from(err: &str) -> Self {
        RttError::Other(err.to_string())
    }
}

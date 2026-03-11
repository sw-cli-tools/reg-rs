//! reg-rs (regress) - Regression Test Tool
#![deny(warnings, missing_docs)]

use std::env;
use std::path::PathBuf;

/// Default status port for web
pub const DEFAULT_STATUS_PORT: u16 = 4111;
/// Status banner
pub const STATUS_BANNER: &str = "reg-rs Status Server";
/// Test database file extension
pub const TDB_EXTENSION: &str = "tdb";
/// Lock file extension (appended to .tdb path)
pub const LOCK_EXTENSION: &str = "lock";
/// File watcher debounce interval in seconds
pub const FILE_WATCH_DEBOUNCE_SECS: u64 = 5;
/// Template spacer for report alignment
pub const REQUIRED_BLANK: &str = " ";

/// Return the data directory, creating it if needed.
///
/// Resolution order:
/// 1. `REG_RS_DATA_DIR` environment variable (if set)
/// 2. `~/.local/reg-rs/` (default)
/// 3. `./data/` (fallback when HOME is unavailable)
pub fn data_dir() -> PathBuf {
    let dir = if let Ok(custom) = env::var("REG_RS_DATA_DIR") {
        PathBuf::from(custom)
    } else {
        env::var("HOME")
            .map(|h| PathBuf::from(h).join(".local").join("reg-rs"))
            .unwrap_or_else(|_| PathBuf::from("data"))
    };
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_dir_returns_path() {
        // data_dir() should always return a non-empty path
        let dir = data_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_data_dir_ends_with_expected_component() {
        // When REG_RS_DATA_DIR is set (by the test harness), it should be used.
        // When not set, the path should end with "reg-rs" (from ~/.local/reg-rs/).
        let dir = data_dir();
        let last = dir.file_name().unwrap().to_string_lossy();
        // Either "reg-rs" (default or env override) or "data" (HOME fallback)
        assert!(
            last == "reg-rs" || last == "data",
            "unexpected data dir: {}",
            dir.display()
        );
    }

    #[test]
    fn test_data_dir_creates_directory() {
        let dir = data_dir();
        assert!(dir.exists(), "data_dir() should create the directory");
    }
}

/// Argument parsing
pub mod args;
/// Test Builder
pub mod builder;
/// Command Processor
pub mod command;
/// Command Configuration
pub mod config;
/// Test Results Database
pub mod db;
/// Test Differences
pub mod diff;
/// Error types and utilities
pub mod error;
/// Command executor trait for dependency injection
pub mod executor;
/// Test Finder
pub mod finder;
/// Logging
pub mod logging;
/// Test Process
pub mod process;
/// Test Result DB Queries
pub mod queries;
/// Test Result Reports
pub mod reporters;
/// Test Runner
pub mod runner;
/// Test DB Interface
pub mod sqlite;
/// Status
pub mod status;
/// Templates
pub mod templates;
/// Time
pub mod time;
/// Utility
pub mod util;

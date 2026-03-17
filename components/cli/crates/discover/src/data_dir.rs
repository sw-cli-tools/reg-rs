//! Data directory discovery for test files.

use std::env;
use std::path::PathBuf;

use reg_rs_types::constants::{RGT_EXTENSION, TDB_EXTENSION};

/// Return the data directory, creating it if needed.
pub fn data_dir() -> PathBuf {
    let dir = if let Ok(custom) = env::var("REG_RS_DATA_DIR") {
        PathBuf::from(custom)
    } else if PathBuf::from("work/reg-rs").exists() {
        PathBuf::from("work/reg-rs")
    } else if has_test_files(".") {
        PathBuf::from(".")
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

/// Check if a directory contains any `.tdb` or `.rgt` files (non-recursive).
fn has_test_files(path: &str) -> bool {
    std::fs::read_dir(path)
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == TDB_EXTENSION || ext == RGT_EXTENSION)
            })
        })
        .unwrap_or(false)
}

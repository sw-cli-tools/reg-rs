//! .rgt file format — TOML-based test specifications with .out/.err baselines.

use serde::{Deserialize, Serialize};

use reg_rs_types::constants::{ERR_EXTENSION, OUT_EXTENSION};
use reg_rs_types::error::{RegError, Result};

// Re-export path utilities so callers can use rgt::db_path etc.
pub use crate::rgt_util::{db_path, is_rgt_path, tdb_path_for_rgt};

/// A regression test specification parsed from a `.rgt` TOML file.
#[derive(Debug, Deserialize, Serialize)]
pub struct RgtSpec {
    /// Shell command to execute
    pub command: String,
    /// Timeout in seconds (default: 300)
    pub timeout: Option<u64>,
    /// Shell command to preprocess stdout/stderr before diffing
    pub preprocess: Option<String>,
    /// Built-in diff normalization: text, json, lines-unordered
    pub diff_mode: Option<String>,
    /// Expected exit code (if present, compared; if absent, not compared)
    pub exit_code: Option<i32>,
    /// Human-readable description
    pub desc: Option<String>,
    /// Expected behavior description
    pub expects: Option<String>,
    /// Known flakiness notes
    pub flaky_note: Option<String>,
}

/// Parse a `.rgt` TOML file into an `RgtSpec`.
pub fn parse_rgt(path: &str) -> Result<RgtSpec> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| RegError::Other(format!("cannot read {}: {}", path, e)))?;
    toml::from_str(&content).map_err(|e| RegError::Other(format!("cannot parse {}: {}", path, e)))
}

/// Read the companion `.out` file (expected stdout baseline).
pub fn read_baseline_stdout(rgt_path: &str) -> Result<String> {
    let out_path = companion_path(rgt_path, OUT_EXTENSION);
    std::fs::read_to_string(&out_path)
        .map_err(|e| RegError::Other(format!("cannot read {}: {}", out_path, e)))
}

/// Read the companion `.err` file (expected stderr baseline).
/// Returns empty string if the file does not exist.
pub fn read_baseline_stderr(rgt_path: &str) -> Result<String> {
    let err_path = companion_path(rgt_path, ERR_EXTENSION);
    match std::fs::read_to_string(&err_path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(RegError::Other(format!("cannot read {}: {}", err_path, e))),
    }
}

/// Write an `RgtSpec` as a TOML `.rgt` file.
pub fn write_rgt(path: &str, spec: &RgtSpec) -> Result<()> {
    let content = toml::to_string_pretty(spec)
        .map_err(|e| RegError::Other(format!("cannot serialize spec: {}", e)))?;
    std::fs::write(path, content)
        .map_err(|e| RegError::Other(format!("cannot write {}: {}", path, e)))
}

/// Write baseline `.out` and `.err` files alongside an `.rgt` file.
pub fn write_baseline(rgt_path: &str, stdout: &str, stderr: &str) -> Result<()> {
    let out_path = companion_path(rgt_path, OUT_EXTENSION);
    std::fs::write(&out_path, stdout)
        .map_err(|e| RegError::Other(format!("cannot write {}: {}", out_path, e)))?;
    let err_path = companion_path(rgt_path, ERR_EXTENSION);
    if stderr.is_empty() {
        let _ = std::fs::remove_file(&err_path);
    } else {
        std::fs::write(&err_path, stderr)
            .map_err(|e| RegError::Other(format!("cannot write {}: {}", err_path, e)))?;
    }
    Ok(())
}

/// Swap the file extension to produce a companion file path.
fn companion_path(rgt_path: &str, ext: &str) -> String {
    let p = std::path::Path::new(rgt_path);
    p.with_extension(ext).to_string_lossy().to_string()
}

//! .rgt file format — TOML-based test specifications with .out/.err baselines.
//!
//! A `.rgt` file defines a regression test. It is a TOML file with fields for the
//! command to run, optional timeout, preprocessing, and metadata. Companion `.out`
//! and `.err` files store the expected stdout and stderr as raw content.

use serde::{Deserialize, Serialize};

use crate::error::{RegError, Result};

/// File extension for test specification files.
pub const RGT_EXTENSION: &str = "rgt";
/// File extension for expected stdout baseline.
pub const OUT_EXTENSION: &str = "out";
/// File extension for expected stderr baseline.
pub const ERR_EXTENSION: &str = "err";

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
/// Creates `.out` always. Creates `.err` only if stderr is non-empty.
/// Removes stale `.err` if stderr is empty.
pub fn write_baseline(rgt_path: &str, stdout: &str, stderr: &str) -> Result<()> {
    let out_path = companion_path(rgt_path, OUT_EXTENSION);
    std::fs::write(&out_path, stdout)
        .map_err(|e| RegError::Other(format!("cannot write {}: {}", out_path, e)))?;
    let err_path = companion_path(rgt_path, ERR_EXTENSION);
    if stderr.is_empty() {
        // Remove stale .err file if it exists
        let _ = std::fs::remove_file(&err_path);
    } else {
        std::fs::write(&err_path, stderr)
            .map_err(|e| RegError::Other(format!("cannot write {}: {}", err_path, e)))?;
    }
    Ok(())
}

/// Get the `.tdb` cache path for an `.rgt` file.
pub fn tdb_path_for_rgt(rgt_path: &str) -> String {
    companion_path(rgt_path, crate::TDB_EXTENSION)
}

/// Swap the file extension to produce a companion file path.
fn companion_path(rgt_path: &str, ext: &str) -> String {
    let p = std::path::Path::new(rgt_path);
    p.with_extension(ext).to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_rgt_minimal() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.rgt");
        std::fs::write(&path, "command = \"echo hello\"\n").unwrap();
        let spec = parse_rgt(path.to_str().unwrap()).unwrap();
        assert_eq!(spec.command, "echo hello");
        assert!(spec.timeout.is_none());
        assert!(spec.exit_code.is_none());
    }

    #[test]
    fn test_parse_rgt_full() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.rgt");
        std::fs::write(
            &path,
            r#"
command = "echo hello"
timeout = 10
preprocess = "sed 's/x/y/g'"
diff_mode = "json"
exit_code = 0
desc = "A test"
expects = "hello"
flaky_note = "None"
"#,
        )
        .unwrap();
        let spec = parse_rgt(path.to_str().unwrap()).unwrap();
        assert_eq!(spec.command, "echo hello");
        assert_eq!(spec.timeout, Some(10));
        assert_eq!(spec.exit_code, Some(0));
        assert_eq!(spec.desc.as_deref(), Some("A test"));
    }

    #[test]
    fn test_parse_rgt_missing_command_fails() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.rgt");
        std::fs::write(&path, "timeout = 10\n").unwrap();
        assert!(parse_rgt(path.to_str().unwrap()).is_err());
    }

    #[test]
    fn test_read_baseline_stdout() {
        let dir = TempDir::new().unwrap();
        let rgt = dir.path().join("test.rgt");
        let out = dir.path().join("test.out");
        std::fs::write(&rgt, "").unwrap();
        std::fs::write(&out, "hello world\n").unwrap();
        let content = read_baseline_stdout(rgt.to_str().unwrap()).unwrap();
        assert_eq!(content, "hello world\n");
    }

    #[test]
    fn test_read_baseline_stderr_missing_returns_empty() {
        let dir = TempDir::new().unwrap();
        let rgt = dir.path().join("test.rgt");
        std::fs::write(&rgt, "").unwrap();
        // No .err file
        let content = read_baseline_stderr(rgt.to_str().unwrap()).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_read_baseline_stderr_exists() {
        let dir = TempDir::new().unwrap();
        let rgt = dir.path().join("test.rgt");
        let err = dir.path().join("test.err");
        std::fs::write(&rgt, "").unwrap();
        std::fs::write(&err, "warning\n").unwrap();
        let content = read_baseline_stderr(rgt.to_str().unwrap()).unwrap();
        assert_eq!(content, "warning\n");
    }

    #[test]
    fn test_write_rgt_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.rgt");
        let spec = RgtSpec {
            command: "echo hello".to_string(),
            timeout: Some(10),
            preprocess: None,
            diff_mode: None,
            exit_code: Some(0),
            desc: Some("A test".to_string()),
            expects: None,
            flaky_note: None,
        };
        write_rgt(path.to_str().unwrap(), &spec).unwrap();
        let parsed = parse_rgt(path.to_str().unwrap()).unwrap();
        assert_eq!(parsed.command, "echo hello");
        assert_eq!(parsed.timeout, Some(10));
        assert_eq!(parsed.exit_code, Some(0));
    }

    #[test]
    fn test_write_baseline_with_stderr() {
        let dir = TempDir::new().unwrap();
        let rgt = dir.path().join("test.rgt");
        write_baseline(rgt.to_str().unwrap(), "out\n", "err\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.path().join("test.out")).unwrap(),
            "out\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("test.err")).unwrap(),
            "err\n"
        );
    }

    #[test]
    fn test_write_baseline_empty_stderr_no_file() {
        let dir = TempDir::new().unwrap();
        let rgt = dir.path().join("test.rgt");
        write_baseline(rgt.to_str().unwrap(), "out\n", "").unwrap();
        assert!(dir.path().join("test.out").exists());
        assert!(!dir.path().join("test.err").exists());
    }

    #[test]
    fn test_tdb_path_for_rgt() {
        assert_eq!(tdb_path_for_rgt("/tmp/test.rgt"), "/tmp/test.tdb");
    }

    #[test]
    fn test_companion_path() {
        assert_eq!(companion_path("/tmp/test.rgt", "out"), "/tmp/test.out");
        assert_eq!(companion_path("test.rgt", "err"), "test.err");
    }
}

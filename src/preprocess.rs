//! Output preprocessing for normalization before diffing.
//!
//! A preprocess command is a shell pipeline that transforms test output
//! (e.g., `jq --sort-keys`, `sed 's/timestamp:.*/timestamp: <MASKED>/'`)
//! to eliminate non-deterministic fields before comparison.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::error::{RegError, Result};

/// Metadata key for the preprocess command in the test database.
pub const PREPROCESS_KEY: &str = "preprocess";

/// Apply a preprocess command to a string by piping it through a shell.
///
/// Returns the original string unchanged if `preprocess` is None or empty.
pub fn apply(input: &str, preprocess: Option<&str>) -> Result<String> {
    let cmd = match preprocess {
        Some(p) if !p.is_empty() => p,
        _ => return Ok(input.to_string()),
    };

    log::debug!("preprocess/apply cmd={}", cmd);

    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| RegError::Other(format!("Failed to spawn preprocess command: {}", e)))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|e| RegError::Other(format!("Failed to write to preprocess stdin: {}", e)))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| RegError::Other(format!("Preprocess command failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RegError::Other(format!(
            "Preprocess command '{}' exited with {}: {}",
            cmd,
            output.status,
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_none() {
        let result = apply("hello", None).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_apply_empty() {
        let result = apply("hello", Some("")).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_apply_cat() {
        let result = apply("hello world", Some("cat")).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_apply_sort() {
        let result = apply("banana\napple\ncherry\n", Some("sort")).unwrap();
        assert_eq!(result, "apple\nbanana\ncherry\n");
    }

    #[test]
    fn test_apply_sed() {
        let input = "timestamp: 2026-03-11T12:00:00\ndata: hello\n";
        let result = apply(input, Some("sed 's/timestamp:.*/timestamp: <MASKED>/'")).unwrap();
        assert!(result.contains("timestamp: <MASKED>"));
        assert!(result.contains("data: hello"));
    }

    #[test]
    fn test_apply_invalid_command() {
        let result = apply("hello", Some("nonexistent_command_xyz"));
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_pipeline() {
        let result = apply("b\na\nc\n", Some("sort | head -2")).unwrap();
        assert_eq!(result, "a\nb\n");
    }
}

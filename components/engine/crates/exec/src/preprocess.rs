//! Output preprocessing for normalization before diffing.

use std::io::Write;
use std::process::{Command, Stdio};

use reg_rs_types::error::{RegError, Result};

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

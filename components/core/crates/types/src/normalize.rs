//! Built-in output normalization for diff modes.
//!
//! Unlike external preprocessing which pipes through shell commands,
//! normalization is done in pure Rust with no external dependencies.

use std::fmt;
use std::str::FromStr;

use crate::error::{RegError, Result};

/// Built-in diff normalization modes.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum DiffMode {
    /// Plain text comparison (no normalization)
    #[default]
    Text,
    /// JSON-aware comparison: sort keys, normalize whitespace
    Json,
    /// Line-order-insensitive comparison: sort lines before diffing
    LinesUnordered,
}

impl fmt::Display for DiffMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Json => write!(f, "json"),
            Self::LinesUnordered => write!(f, "lines-unordered"),
        }
    }
}

impl FromStr for DiffMode {
    type Err = RegError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "lines-unordered" => Ok(Self::LinesUnordered),
            other => Err(RegError::Config(format!(
                "Unknown diff mode '{}'. Valid modes: text, json, lines-unordered",
                other
            ))),
        }
    }
}

/// Normalize a string according to the diff mode.
pub fn apply(input: &str, mode: &DiffMode) -> Result<String> {
    match mode {
        DiffMode::Text => Ok(input.to_string()),
        DiffMode::Json => normalize_json(input),
        DiffMode::LinesUnordered => Ok(sort_lines(input)),
    }
}

/// Sort lines alphabetically for order-insensitive comparison.
fn sort_lines(input: &str) -> String {
    let mut lines: Vec<&str> = input.lines().collect();
    lines.sort();
    let mut result = lines.join("\n");
    if input.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Attempt JSON normalization; return input unchanged if it's empty.
fn normalize_json(input: &str) -> Result<String> {
    if input.trim().is_empty() {
        return Ok(input.to_string());
    }

    let value: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| RegError::Other(format!("diff-mode json: output is not valid JSON: {}", e)))?;

    let mut normalized = serde_json::to_string_pretty(&value)
        .map_err(|e| RegError::Other(format!("diff-mode json: failed to serialize: {}", e)))?;
    normalized.push('\n');
    Ok(normalized)
}

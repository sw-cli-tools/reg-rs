//! Built-in output normalization for diff modes.
//!
//! Unlike [`crate::preprocess`] which pipes through external shell commands,
//! normalization is done in pure Rust with no external dependencies.

use std::fmt;
use std::str::FromStr;

use crate::error::{RegError, Result};

/// Metadata key for the diff mode in the test database.
pub const DIFF_MODE_KEY: &str = "diff_mode";

/// Built-in diff normalization modes.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum DiffMode {
    /// Plain text comparison (no normalization)
    #[default]
    Text,
    /// JSON-aware comparison: sort keys, normalize whitespace
    Json,
}

impl fmt::Display for DiffMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Json => write!(f, "json"),
        }
    }
}

impl FromStr for DiffMode {
    type Err = RegError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(RegError::Config(format!(
                "Unknown diff mode '{}'. Valid modes: text, json",
                other
            ))),
        }
    }
}

/// Normalize a string according to the diff mode.
///
/// - `Text`: returns input unchanged.
/// - `Json`: parses as JSON, sorts keys (via serde_json's BTreeMap), re-serializes pretty.
///   Returns an error if the input is not valid JSON.
pub fn apply(input: &str, mode: &DiffMode) -> Result<String> {
    match mode {
        DiffMode::Text => Ok(input.to_string()),
        DiffMode::Json => normalize_json(input),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_mode_passthrough() {
        let result = apply("hello world", &DiffMode::Text).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_text_mode_empty() {
        let result = apply("", &DiffMode::Text).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_json_sorts_keys() {
        let input = r#"{"z":1,"a":2,"m":3}"#;
        let result = apply(input, &DiffMode::Json).unwrap();
        // Keys should be sorted: a, m, z
        let a_pos = result.find("\"a\"").unwrap();
        let m_pos = result.find("\"m\"").unwrap();
        let z_pos = result.find("\"z\"").unwrap();
        assert!(a_pos < m_pos);
        assert!(m_pos < z_pos);
    }

    #[test]
    fn test_json_nested_sorts() {
        let input = r#"{"outer":{"z":1,"a":2}}"#;
        let result = apply(input, &DiffMode::Json).unwrap();
        let a_pos = result.find("\"a\"").unwrap();
        let z_pos = result.find("\"z\"").unwrap();
        assert!(a_pos < z_pos);
    }

    #[test]
    fn test_json_invalid_returns_error() {
        let result = apply("not json at all", &DiffMode::Json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not valid JSON"));
    }

    #[test]
    fn test_json_empty_passthrough() {
        let result = apply("", &DiffMode::Json).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_json_whitespace_only_passthrough() {
        let result = apply("  \n  ", &DiffMode::Json).unwrap();
        assert_eq!(result, "  \n  ");
    }

    #[test]
    fn test_json_deterministic() {
        let input1 = r#"{"b": 2, "a": 1}"#;
        let input2 = r#"{"a":1,"b":2}"#;
        let result1 = apply(input1, &DiffMode::Json).unwrap();
        let result2 = apply(input2, &DiffMode::Json).unwrap();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_display_text() {
        assert_eq!(DiffMode::Text.to_string(), "text");
    }

    #[test]
    fn test_display_json() {
        assert_eq!(DiffMode::Json.to_string(), "json");
    }

    #[test]
    fn test_from_str_valid() {
        assert_eq!("text".parse::<DiffMode>().unwrap(), DiffMode::Text);
        assert_eq!("json".parse::<DiffMode>().unwrap(), DiffMode::Json);
    }

    #[test]
    fn test_from_str_invalid() {
        assert!("xml".parse::<DiffMode>().is_err());
    }

    #[test]
    fn test_default_is_text() {
        assert_eq!(DiffMode::default(), DiffMode::Text);
    }
}

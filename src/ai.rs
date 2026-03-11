//! AI-powered command generation using the Claude API.
//!
//! Translates natural language descriptions into shell commands
//! for regression test creation.

use std::env;

use serde_json::json;

use crate::error::{RegError, Result};

/// Environment variable for the Anthropic API key
const API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

/// Default model to use for command generation
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Environment variable to override the model
const MODEL_ENV: &str = "REG_RS_AI_MODEL";

/// Claude API endpoint
const API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Generate a shell command from a natural language description.
///
/// Calls the Claude API with a prompt that asks for a single shell command.
/// Optionally includes context (e.g., `--help` output) and existing test
/// commands to guide generation.
/// Returns the generated command string.
pub(crate) fn generate_command(
    description: &str,
    context: Option<&str>,
    existing_tests: &[String],
) -> Result<String> {
    let api_key = env::var(API_KEY_ENV).map_err(|_| {
        RegError::Config(format!(
            "{} environment variable not set. Get an API key at https://console.anthropic.com/",
            API_KEY_ENV
        ))
    })?;

    let model = env::var(MODEL_ENV).unwrap_or_else(|_| DEFAULT_MODEL.to_string());

    log::info!(
        "ai/generate_command model={}, description={}",
        model,
        description
    );

    let prompt = build_prompt(description, context, existing_tests);
    let body = build_request_body(&model, &prompt);

    let response = ureq::post(API_URL)
        .set("x-api-key", &api_key)
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .send_json(&body)
        .map_err(|e| RegError::Other(format!("Claude API request failed: {}", e)))?;

    let response_body: serde_json::Value = response
        .into_json()
        .map_err(|e| RegError::Other(format!("Failed to parse Claude API response: {}", e)))?;

    extract_command(&response_body)
}

/// Build the system+user prompt for command generation
fn build_prompt(description: &str, context: Option<&str>, existing_tests: &[String]) -> String {
    let mut prompt = String::from(
        "You are a shell command generator for a regression testing tool. \
         The user will describe what they want to test, and you must respond \
         with ONLY a single shell command (no explanation, no markdown, no backticks). \
         The command should be something that produces deterministic output \
         suitable for regression testing.\n\n",
    );

    if let Some(ctx) = context {
        prompt.push_str(&format!(
            "Context (tool help or reference output):\n```\n{}\n```\n\n",
            ctx
        ));
    }

    if !existing_tests.is_empty() {
        prompt.push_str("Existing test commands (follow similar patterns):\n");
        for cmd in existing_tests {
            prompt.push_str(&format!("  - {}\n", cmd));
        }
        prompt.push('\n');
    }

    prompt.push_str(&format!("User description: {}", description));
    prompt
}

/// Build the JSON request body for the Claude API
fn build_request_body(model: &str, prompt: &str) -> serde_json::Value {
    json!({
        "model": model,
        "max_tokens": 256,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    })
}

/// Extract the command text from a Claude API response
fn extract_command(response: &serde_json::Value) -> Result<String> {
    let command = response["content"]
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(|block| block["text"].as_str())
        .ok_or_else(|| {
            RegError::Other(format!(
                "Unexpected Claude API response format: {}",
                response
            ))
        })?;

    let command = command.trim().to_string();

    if command.is_empty() {
        return Err(RegError::Other(
            "Claude API returned an empty command".to_string(),
        ));
    }

    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt_includes_description() {
        let prompt = build_prompt("list files in current directory", None, &[]);
        assert!(prompt.contains("list files in current directory"));
        assert!(prompt.contains("shell command generator"));
    }

    #[test]
    fn test_build_prompt_includes_context() {
        let prompt = build_prompt("test help output", Some("Usage: mytool [OPTIONS]"), &[]);
        assert!(prompt.contains("Usage: mytool [OPTIONS]"));
        assert!(prompt.contains("Context"));
    }

    #[test]
    fn test_build_prompt_includes_existing_tests() {
        let existing = vec!["echo hello".to_string(), "ls -la".to_string()];
        let prompt = build_prompt("test something", None, &existing);
        assert!(prompt.contains("echo hello"));
        assert!(prompt.contains("ls -la"));
        assert!(prompt.contains("Existing test commands"));
    }

    #[test]
    fn test_build_request_body_structure() {
        let body = build_request_body("claude-sonnet-4-20250514", "test prompt");
        assert_eq!(body["model"], "claude-sonnet-4-20250514");
        assert_eq!(body["max_tokens"], 256);
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "test prompt");
    }

    #[test]
    fn test_extract_command_valid_response() {
        let response = json!({
            "content": [
                {
                    "type": "text",
                    "text": "ls -la"
                }
            ]
        });
        let command = extract_command(&response).unwrap();
        assert_eq!(command, "ls -la");
    }

    #[test]
    fn test_extract_command_trims_whitespace() {
        let response = json!({
            "content": [
                {
                    "type": "text",
                    "text": "  echo hello  \n"
                }
            ]
        });
        let command = extract_command(&response).unwrap();
        assert_eq!(command, "echo hello");
    }

    #[test]
    fn test_extract_command_empty_response() {
        let response = json!({
            "content": [
                {
                    "type": "text",
                    "text": ""
                }
            ]
        });
        assert!(extract_command(&response).is_err());
    }

    #[test]
    fn test_extract_command_malformed_response() {
        let response = json!({"error": "bad request"});
        assert!(extract_command(&response).is_err());
    }

    #[test]
    fn test_generate_command_missing_api_key() {
        // Ensure the env var is not set for this test
        unsafe { env::remove_var(API_KEY_ENV) };
        let result = generate_command("list files", None, &[]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains(API_KEY_ENV));
    }
}

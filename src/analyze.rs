//! AI-powered failure analysis using the Claude API.
//!
//! Sends test failure details (original output, latest output, diff) to the
//! Claude API for triage: is this a real regression, a flaky test, or a
//! baseline that needs updating?

use std::env;

use serde_json::json;

use crate::db;
use crate::diff::RegressionType;
use crate::error::{RegError, Result};
use crate::finder;
use crate::runner::TestResults;

/// Environment variable for the Anthropic API key
const API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

/// Default model for analysis (uses a capable model for reasoning)
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Environment variable to override the model
const MODEL_ENV: &str = "REG_RS_AI_MODEL";

/// Claude API endpoint
const API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Analyze failures for tests matching a pattern.
pub fn analyze_failures(pattern: &str) -> Result<()> {
    log::info!("analyze/analyze_failures pattern={}", pattern);

    let tests = finder::discover(pattern.to_string())?;
    if tests.found.is_empty() {
        eprintln!(
            "warning: no tests matched pattern '{}' in {}",
            pattern,
            tests.data_dir.display()
        );
        return Ok(());
    }

    for test in &tests.found {
        analyze_one(test)?;
    }
    Ok(())
}

/// Analyze a single test's failure.
fn analyze_one(db_name: &str) -> Result<()> {
    log::info!("analyze/analyze_one {}", db_name);

    let diff_count = db::count_differences(db_name)?;
    if diff_count == 0 {
        eprintln!("  {} — no differences (passed)", db_name);
        return Ok(());
    }

    let original = db::read_original_results(db_name)?;
    let latest = db::read_latest_results(db_name)?;
    let differences = db::read_differences(db_name)?;

    let prompt = build_analysis_prompt(&original, &latest, &differences);
    let analysis = call_api(&prompt)?;

    eprintln!("\n--- Analysis: {} ---", db_name);
    eprintln!("{}", analysis);
    eprintln!("---\n");

    Ok(())
}

/// Build the prompt for failure analysis.
fn build_analysis_prompt(
    original: &TestResults,
    latest: &TestResults,
    differences: &[(String, String)],
) -> String {
    let mut diff_text = String::new();
    for (type_code, chunk) in differences {
        let label = RegressionType::display_label(type_code).unwrap_or("unchanged");
        diff_text.push_str(&format!("  {}: {}\n", label, chunk));
    }

    format!(
        "You are analyzing a regression test failure for a CLI tool called reg-rs.\n\
         \n\
         ## Test Info\n\
         - Test: {name}\n\
         - Command: `{command}`\n\
         \n\
         ## Original (baseline) output\n\
         - Exit code: {orig_exit}\n\
         - stdout:\n```\n{orig_stdout}\n```\n\
         - stderr:\n```\n{orig_stderr}\n```\n\
         \n\
         ## Latest (current) output\n\
         - Exit code: {latest_exit}\n\
         - stdout:\n```\n{latest_stdout}\n```\n\
         - stderr:\n```\n{latest_stderr}\n```\n\
         \n\
         ## Differences\n\
         {diffs}\n\
         \n\
         ## Your analysis\n\
         Determine:\n\
         1. Is this a **real regression** (behavior change that needs investigation)?\n\
         2. Is this a **flaky test** (non-deterministic output like timestamps, temp paths, ordering)?\n\
         3. Should the **baseline be updated** (expected change, e.g., version bump)?\n\
         \n\
         For each, explain why. If flaky, suggest how to make it robust \
         (e.g., preprocess command to strip volatile fields, or a diff-mode). \
         Keep your response concise (under 200 words).",
        name = original.name,
        command = original.command,
        orig_exit = original.exit_code,
        orig_stdout = original.stdout,
        orig_stderr = original.stderr,
        latest_exit = latest.exit_code,
        latest_stdout = latest.stdout,
        latest_stderr = latest.stderr,
        diffs = diff_text,
    )
}

/// Call the Claude API with the analysis prompt.
fn call_api(prompt: &str) -> Result<String> {
    let api_key = env::var(API_KEY_ENV).map_err(|_| {
        RegError::Config(format!(
            "{} environment variable not set. Get an API key at https://console.anthropic.com/",
            API_KEY_ENV
        ))
    })?;

    let model = env::var(MODEL_ENV).unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    log::info!("analyze/call_api model={}", model);

    let body = json!({
        "model": model,
        "max_tokens": 1024,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let response = ureq::post(API_URL)
        .set("x-api-key", &api_key)
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .send_json(&body)
        .map_err(|e| RegError::Other(format!("Claude API request failed: {}", e)))?;

    let response_body: serde_json::Value = response
        .into_json()
        .map_err(|e| RegError::Other(format!("Failed to parse Claude API response: {}", e)))?;

    let text = response_body["content"]
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(|block| block["text"].as_str())
        .ok_or_else(|| {
            RegError::Other(format!(
                "Unexpected Claude API response format: {}",
                response_body
            ))
        })?;

    Ok(text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_analysis_prompt_contains_test_info() {
        let original = TestResults {
            name: "my_test".to_string(),
            command: "echo hello".to_string(),
            time_created: "2026-01-01".to_string(),
            exit_code: 0,
            stdout: "hello\n".to_string(),
            stderr: "".to_string(),
        };
        let latest = TestResults {
            name: "my_test".to_string(),
            command: "echo hello".to_string(),
            time_created: "2026-01-02".to_string(),
            exit_code: 0,
            stdout: "hello world\n".to_string(),
            stderr: "".to_string(),
        };
        let differences = vec![
            ("7".to_string(), "hello".to_string()),
            ("6".to_string(), "hello world".to_string()),
        ];
        let prompt = build_analysis_prompt(&original, &latest, &differences);
        assert!(prompt.contains("my_test"));
        assert!(prompt.contains("echo hello"));
        assert!(prompt.contains("hello world"));
        assert!(prompt.contains("stdout remove"));
        assert!(prompt.contains("stdout add"));
    }

    #[test]
    fn test_build_analysis_prompt_handles_exit_code_diff() {
        let original = TestResults {
            name: "exit_test".to_string(),
            command: "false".to_string(),
            time_created: "2026-01-01".to_string(),
            exit_code: 0,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        let latest = TestResults {
            name: "exit_test".to_string(),
            command: "false".to_string(),
            time_created: "2026-01-02".to_string(),
            exit_code: 1,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        let differences = vec![
            ("2".to_string(), "0".to_string()),
            ("1".to_string(), "1".to_string()),
        ];
        let prompt = build_analysis_prompt(&original, &latest, &differences);
        assert!(prompt.contains("Exit code: 0"));
        assert!(prompt.contains("Exit code: 1"));
        assert!(prompt.contains("Expected exit code"));
        assert!(prompt.contains("Actual exit code"));
    }

    #[test]
    fn test_call_api_missing_key() {
        unsafe { env::remove_var(API_KEY_ENV) };
        let result = call_api("test prompt");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains(API_KEY_ENV));
    }
}

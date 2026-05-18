//! AI-powered failure analysis using the Claude API.
//!
//! Sends test failure details (original output, latest output, diff) to the
//! Claude API for triage: is this a real regression, a flaky test, or a
//! baseline that needs updating?

use std::env;

use serde_json::json;

use reg_rs_store::db;
use reg_rs_store::db_ops;
use reg_rs_types::error::{RegError, Result};
use reg_rs_types::types::{RegressionType, TestResults};

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
    log::info!("analyze/analyze_failures pattern={pattern}");

    let tests = reg_rs_discover::finder::discover(pattern.to_string())?;
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
    log::info!("analyze/analyze_one {db_name}");

    let diff_count = db_ops::count_differences(db_name)?;
    if diff_count == 0 {
        eprintln!("  {db_name} — no differences (passed)");
        return Ok(());
    }

    let original = db::read_original_results(db_name)?;
    let latest = db::read_latest_results(db_name)?;
    let differences = db_ops::read_differences(db_name)?;

    let prompt = build_analysis_prompt(&original, &latest, &differences);
    let analysis = call_api(&prompt)?;

    eprintln!("\n--- Analysis: {db_name} ---");
    eprintln!("{analysis}");
    eprintln!("---\n");

    Ok(())
}

/// Format difference entries as labeled text lines.
pub fn format_differences(differences: &[(String, String)]) -> String {
    let mut diff_text = String::new();
    for (type_code, chunk) in differences {
        let label = RegressionType::display_label(type_code).unwrap_or("unchanged");
        diff_text.push_str(&format!("  {label}: {chunk}\n"));
    }
    diff_text
}

/// Build the prompt for failure analysis.
pub fn build_analysis_prompt(
    original: &TestResults,
    latest: &TestResults,
    differences: &[(String, String)],
) -> String {
    let diff_text = format_differences(differences);

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
pub fn call_api(prompt: &str) -> Result<String> {
    let api_key = env::var(API_KEY_ENV).map_err(|_| {
        RegError::Config(format!(
            "{API_KEY_ENV} environment variable not set. Get an API key at https://console.anthropic.com/"
        ))
    })?;

    let model = env::var(MODEL_ENV).unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    log::info!("analyze/call_api model={model}");

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
        .map_err(|e| RegError::Other(format!("Claude API request failed: {e}")))?;

    let response_body: serde_json::Value = response
        .into_json()
        .map_err(|e| RegError::Other(format!("Failed to parse Claude API response: {e}")))?;

    let text = response_body["content"]
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(|block| block["text"].as_str())
        .ok_or_else(|| {
            RegError::Other(format!(
                "Unexpected Claude API response format: {response_body}"
            ))
        })?;

    Ok(text.trim().to_string())
}

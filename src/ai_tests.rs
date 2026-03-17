use super::*;
use serde_json::json;

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

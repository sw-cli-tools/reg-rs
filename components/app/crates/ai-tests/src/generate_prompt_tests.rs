use reg_rs_ai::generate::{build_prompt, build_request_body};

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

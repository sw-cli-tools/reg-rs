use reg_rs_ai::generate::{extract_command, generate_command};
use serde_json::json;

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
    unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
    let result = generate_command("list files", None, &[]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("ANTHROPIC_API_KEY"));
}

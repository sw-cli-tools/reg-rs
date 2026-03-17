use reg_rs_ai::analyze::{build_analysis_prompt, call_api};
use reg_rs_types::types::TestResults;

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
    unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
    let result = call_api("test prompt");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("ANTHROPIC_API_KEY"));
}

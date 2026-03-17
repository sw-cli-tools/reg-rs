use reg_rs_exec::executor::{CommandExecutor, RealCommandExecutor};

use super::mock::MockCommandExecutor;

#[test]
fn test_mock_executor_success() {
    let executor = MockCommandExecutor::success("hello world\n");
    let (exit, stderr, stdout) = executor.exec("echo hello world").unwrap();
    assert_eq!(exit, 0);
    assert_eq!(stderr, "");
    assert_eq!(stdout, "hello world\n");
}

#[test]
fn test_mock_executor_failure() {
    let executor = MockCommandExecutor::failure(1, "command not found");
    let (exit, stderr, stdout) = executor.exec("nonexistent").unwrap();
    assert_eq!(exit, 1);
    assert_eq!(stderr, "command not found");
    assert_eq!(stdout, "");
}

#[test]
fn test_mock_executor_multiple_responses() {
    let executor = MockCommandExecutor::new(vec![
        (0, String::new(), "first\n".to_string()),
        (0, String::new(), "second\n".to_string()),
    ]);

    let (_, _, stdout) = executor.exec("first").unwrap();
    assert_eq!(stdout, "first\n");

    let (_, _, stdout) = executor.exec("second").unwrap();
    assert_eq!(stdout, "second\n");
}

#[test]
fn test_real_executor() {
    let executor = RealCommandExecutor::new();
    let (code, _stderr, stdout) = executor.exec("echo hello").unwrap();
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "hello");
}

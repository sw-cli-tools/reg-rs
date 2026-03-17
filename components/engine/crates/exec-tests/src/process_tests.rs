use reg_rs_exec::process;
use std::time::Duration;

#[test]
fn test_exec_captures_stdout() {
    let (code, stderr, stdout) =
        process::exec_with_timeout("echo hello".into(), Duration::from_secs(5))
            .expect("failed to exec echo hello");
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "hello");
    assert_eq!(stderr, "");
}

#[test]
fn test_exec_captures_stderr() {
    let (code, stderr, _stdout) =
        process::exec_with_timeout("echo oops >&2".into(), Duration::from_secs(5))
            .expect("failed to exec stderr command");
    assert_eq!(code, 0);
    assert_eq!(stderr.trim(), "oops");
}

#[test]
fn test_exec_captures_exit_code() {
    let (code, _stderr, _stdout) =
        process::exec_with_timeout("exit 42".into(), Duration::from_secs(5))
            .expect("failed to exec exit 42");
    assert_eq!(code, 42);
}

#[test]
fn test_exec_timeout_kills_long_running_command() {
    let result = process::exec_with_timeout("sleep 60".into(), Duration::from_millis(200));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("timed out"),
        "error should mention timeout: {}",
        err
    );
}

#[test]
fn test_exec_fast_command_within_timeout() {
    let result = process::exec_with_timeout("echo fast".into(), Duration::from_secs(5));
    assert!(result.is_ok());
    let (code, _stderr, stdout) = result.expect("failed to exec fast command");
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "fast");
}

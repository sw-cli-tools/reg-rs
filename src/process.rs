use std::io::Read;
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

use crate::error::RegError;

/// Default command timeout (5 minutes)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// Run a test capturing outputs and exit code.
///
/// Commands are killed after the default timeout (5 minutes) to prevent
/// hanging tests from blocking indefinitely and holding database locks.
pub fn exec(command: String) -> crate::error::Result<(i32, String, String)> {
    exec_with_timeout(command, DEFAULT_TIMEOUT)
}

/// Run a test with a specific timeout, capturing outputs and exit code.
pub fn exec_with_timeout(
    command: String,
    timeout: Duration,
) -> crate::error::Result<(i32, String, String)> {
    log::info!(
        "process/exec command: {} (timeout: {:?})",
        &command,
        timeout
    );
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            RegError::CommandExecution(format!("failed to execute '{}': {}", command, e))
        })?;

    // Read stdout/stderr in separate threads to avoid pipe deadlocks
    let mut child_stdout = child.stdout.take();
    let mut child_stderr = child.stderr.take();

    let stdout_handle = std::thread::spawn(move || -> String {
        let mut buf = String::new();
        if let Some(ref mut out) = child_stdout {
            let _ = out.read_to_string(&mut buf);
        }
        buf
    });
    let stderr_handle = std::thread::spawn(move || -> String {
        let mut buf = String::new();
        if let Some(ref mut err) = child_stderr {
            let _ = err.read_to_string(&mut buf);
        }
        buf
    });

    // Wait for the child process with a timeout.
    // Capture PID before moving child into the wait thread so we can
    // kill by PID on timeout (joining the thread would block otherwise).
    let child_pid = child.id();
    let (tx, rx) = mpsc::channel();
    let wait_handle = std::thread::spawn(move || {
        let result = child.wait();
        let _ = tx.send(result);
        child
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(status)) => {
            let stdout = stdout_handle.join().unwrap_or_default();
            let stderr = stderr_handle.join().unwrap_or_default();
            let status_code = status.code().unwrap_or(-1);

            log::debug!("status: {:#?} status_code:{}", status, status_code);
            log::debug!("stdout:\n{}", &stdout);
            log::debug!("stderr:\n{}", &stderr);

            Ok((status_code, stderr, stdout))
        }
        Ok(Err(e)) => Err(RegError::CommandExecution(format!(
            "failed waiting for '{}': {}",
            command, e
        ))),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Kill the child process by PID first so the wait thread unblocks
            let _ = Command::new("kill")
                .args(["-9", &child_pid.to_string()])
                .output();
            // Now join the wait thread (child.wait() will return quickly)
            let _ = wait_handle.join();
            Err(RegError::CommandExecution(format!(
                "command timed out after {:?}: '{}'",
                timeout, command
            )))
        }
        Err(e) => Err(RegError::CommandExecution(format!(
            "channel error waiting for '{}': {}",
            command, e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_captures_stdout() {
        let (code, stderr, stdout) =
            exec_with_timeout("echo hello".into(), Duration::from_secs(5)).unwrap();
        assert_eq!(code, 0);
        assert_eq!(stdout.trim(), "hello");
        assert_eq!(stderr, "");
    }

    #[test]
    fn test_exec_captures_stderr() {
        let (code, stderr, _stdout) =
            exec_with_timeout("echo oops >&2".into(), Duration::from_secs(5)).unwrap();
        assert_eq!(code, 0);
        assert_eq!(stderr.trim(), "oops");
    }

    #[test]
    fn test_exec_captures_exit_code() {
        let (code, _stderr, _stdout) =
            exec_with_timeout("exit 42".into(), Duration::from_secs(5)).unwrap();
        assert_eq!(code, 42);
    }

    #[test]
    fn test_exec_timeout_kills_long_running_command() {
        let result = exec_with_timeout("sleep 60".into(), Duration::from_millis(200));
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
        let result = exec_with_timeout("echo fast".into(), Duration::from_secs(5));
        assert!(result.is_ok());
        let (code, _stderr, stdout) = result.unwrap();
        assert_eq!(code, 0);
        assert_eq!(stdout.trim(), "fast");
    }
}

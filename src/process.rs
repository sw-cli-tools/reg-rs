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
pub fn exec(command: String) -> Result<(i32, String, String), Box<dyn std::error::Error>> {
    exec_with_timeout(command, DEFAULT_TIMEOUT)
}

/// Run a test with a specific timeout, capturing outputs and exit code.
pub fn exec_with_timeout(
    command: String,
    timeout: Duration,
) -> Result<(i32, String, String), Box<dyn std::error::Error>> {
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

    // Wait for the child process with a timeout
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

            println!("status: {:#?} status_code:{}", status, status_code);
            println!("stdout:\n{}", &stdout);
            println!("stderr:\n{}", &stderr);

            Ok((status_code, stderr, stdout))
        }
        Ok(Err(e)) => Err(Box::new(RegError::CommandExecution(format!(
            "failed waiting for '{}': {}",
            command, e
        )))),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Kill the child process
            if let Ok(mut child) = wait_handle.join() {
                let _ = child.kill();
                let _ = child.wait();
            }
            Err(Box::new(RegError::CommandExecution(format!(
                "command timed out after {:?}: '{}'",
                timeout, command
            ))))
        }
        Err(e) => Err(Box::new(RegError::CommandExecution(format!(
            "channel error waiting for '{}': {}",
            command, e
        )))),
    }
}

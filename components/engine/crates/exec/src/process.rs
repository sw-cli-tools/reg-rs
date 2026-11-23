use std::process::Command;
use std::time::{Duration, Instant};

use reg_rs_types::error::RegError;

use crate::process_io::{collect_output, kill_child};

/// Default command timeout (5 minutes)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// How often to check whether a running command has exited
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Run a test capturing outputs and exit code.
pub fn exec(command: String) -> reg_rs_types::error::Result<(i32, String, String)> {
    exec_with_timeout(command, DEFAULT_TIMEOUT)
}

/// Run a test with a specific timeout, capturing outputs and exit code.
pub fn exec_with_timeout(
    command: String,
    timeout: Duration,
) -> reg_rs_types::error::Result<(i32, String, String)> {
    log::info!("process/exec command: {} (timeout: {:?})", command, timeout);
    let mut child = spawn_shell(&command)?;
    let (stdout_handle, stderr_handle) = spawn_output_readers(&mut child);
    match wait_until(&mut child, timeout) {
        Ok(Some(status)) => collect_output(status, stdout_handle, stderr_handle),
        Ok(None) => {
            kill_child(&mut child);
            Err(RegError::CommandExecution(format!(
                "command timed out after {timeout:?}: '{command}'"
            )))
        }
        Err(e) => Err(RegError::CommandExecution(format!(
            "failed waiting for '{command}': {e}"
        ))),
    }
}

/// Poll the child until it exits or the timeout elapses (`None` on timeout).
fn wait_until(
    child: &mut std::process::Child,
    timeout: Duration,
) -> std::io::Result<Option<std::process::ExitStatus>> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

/// Spawn a shell process for the given command string.
fn spawn_shell(command: &str) -> reg_rs_types::error::Result<std::process::Child> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| RegError::CommandExecution(format!("failed to execute '{command}': {e}")))
}

/// Spawn threads to read stdout and stderr without pipe deadlocks.
fn spawn_output_readers(
    child: &mut std::process::Child,
) -> (
    std::thread::JoinHandle<String>,
    std::thread::JoinHandle<String>,
) {
    use std::io::Read;
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
    (stdout_handle, stderr_handle)
}

use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

use reg_rs_types::error::RegError;

use crate::process_io::{collect_output, kill_child};

/// Default command timeout (5 minutes)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// Run a test capturing outputs and exit code.
pub fn exec(command: String) -> reg_rs_types::error::Result<(i32, String, String)> {
    exec_with_timeout(command, DEFAULT_TIMEOUT)
}

/// Run a test with a specific timeout, capturing outputs and exit code.
pub fn exec_with_timeout(
    command: String,
    timeout: Duration,
) -> reg_rs_types::error::Result<(i32, String, String)> {
    log::info!(
        "process/exec command: {} (timeout: {:?})",
        &command,
        timeout
    );
    let mut child = spawn_shell(&command)?;
    let (stdout_handle, stderr_handle) = spawn_output_readers(&mut child);
    let child_pid = child.id();
    let (tx, rx) = mpsc::channel();
    let wait_handle = std::thread::spawn(move || {
        let result = child.wait();
        let _ = tx.send(result);
        child
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(status)) => collect_output(status, stdout_handle, stderr_handle),
        Ok(Err(e)) => Err(RegError::CommandExecution(format!(
            "failed waiting for '{command}': {e}"
        ))),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            kill_child(child_pid, wait_handle);
            Err(RegError::CommandExecution(format!(
                "command timed out after {timeout:?}: '{command}'"
            )))
        }
        Err(e) => Err(RegError::CommandExecution(format!(
            "channel error waiting for '{command}': {e}"
        ))),
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

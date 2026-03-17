use std::process::Command;

/// Collect output from reader threads after the process exits successfully.
pub fn collect_output(
    status: std::process::ExitStatus,
    stdout_handle: std::thread::JoinHandle<String>,
    stderr_handle: std::thread::JoinHandle<String>,
) -> reg_rs_types::error::Result<(i32, String, String)> {
    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();
    let status_code = status.code().unwrap_or(-1);

    log::debug!("status: {:#?} status_code:{}", status, status_code);
    log::debug!("stdout:\n{}", &stdout);
    log::debug!("stderr:\n{}", &stderr);

    Ok((status_code, stderr, stdout))
}

/// Kill a child process by PID and join its wait thread.
pub fn kill_child(child_pid: u32, wait_handle: std::thread::JoinHandle<std::process::Child>) {
    let _ = Command::new("kill")
        .args(["-9", &child_pid.to_string()])
        .output();
    let _ = wait_handle.join();
}

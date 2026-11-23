/// Collect output from reader threads after the process exits successfully.
pub fn collect_output(
    status: std::process::ExitStatus,
    stdout_handle: std::thread::JoinHandle<String>,
    stderr_handle: std::thread::JoinHandle<String>,
) -> reg_rs_types::error::Result<(i32, String, String)> {
    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();
    let status_code = status.code().unwrap_or(-1);

    log::debug!("status: {status:#?} status_code:{status_code}");
    log::debug!("stdout:\n{}", &stdout);
    log::debug!("stderr:\n{}", &stderr);

    Ok((status_code, stderr, stdout))
}

/// Kill a child process and reap it (portable: no external `kill` command).
pub fn kill_child(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

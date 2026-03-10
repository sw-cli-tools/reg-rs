use std::process::Command;

use crate::error::RegError;

/// run a test capturing outputs and exit code
pub fn exec(command: String) -> Result<(i32, String, String), Box<dyn std::error::Error>> {
    log::info!("process/exec command: {}", &command);
    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|e| {
            RegError::CommandExecution(format!("failed to execute '{}': {}", command, e))
        })?;
    let status_code = output.status.code().unwrap_or(-1);

    println!("status: {:#?} status_code:{}", output.status, status_code);
    println!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    let exit = status_code;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok((exit, stderr, stdout))
}

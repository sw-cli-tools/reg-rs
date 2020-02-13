use std::process::Command;

pub fn exec(command: String) -> Result<(i32, String, String), Box<dyn std::error::Error>> {
    let output = Command::new("sh").arg("-c")
                     .arg(command)
                     .output()
        .expect("failed to execute process");
    let status_code = match output.status.code() {
        Some(n) => n,
        _ => -1,
    };

    println!("status: {:#?} status_code:{}", output.status, status_code);
    println!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    let exit = status_code;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok((exit, stderr, stdout))
}

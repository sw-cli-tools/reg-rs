use crate::config;
use crate::db;
use crate::finder;

use super::utils::{format_test_name, read_test_command};

/// List tests matching a pattern with their status
pub fn list(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/list");
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern.clone())?;
    if tests.found.is_empty() {
        eprintln!(
            "no tests matched pattern '{}' in {}",
            pattern,
            tests.data_dir.display()
        );
        return Ok(());
    }
    for test_path in &tests.found {
        let (command, tdb_path) = read_test_command(test_path)?;
        let latest_count = db::count_latest_results(&tdb_path)?;
        let status = if latest_count == 0 {
            "pending"
        } else {
            let diff_count = db::count_differences(&tdb_path)?;
            if diff_count > 0 { "FAIL" } else { "PASS" }
        };
        let name = format_test_name(test_path);
        let cmd_display = if command.len() > 60 {
            format!("{}...", &command[..57])
        } else {
            command
        };
        println!("{:<7} {:<30} {}", status, name, cmd_display);
    }
    println!(
        "---\n{} test(s) matched pattern '{}'",
        tests.found.len(),
        pattern
    );
    log::info!("command/list done");
    Ok(())
}

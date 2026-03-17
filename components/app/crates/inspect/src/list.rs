use reg_rs_config::config::Config;
use reg_rs_store::db_ops;
use reg_rs_store_rgt::rgt;
use reg_rs_store_rgt::rgt_util;
use reg_rs_types::constants::RGT_EXTENSION;
use reg_rs_types::error::Result;

/// List tests matching a pattern with their status
pub fn list(config: &Config) -> Result<()> {
    log::info!("command/list");
    let pattern = config.extract_pattern().to_string();
    let tests = reg_rs_discover::finder::discover(pattern.clone())?;
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
        let latest_count = db_ops::count_latest_results(&tdb_path)?;
        let status = if latest_count == 0 {
            "pending"
        } else {
            let diff_count = db_ops::count_differences(&tdb_path)?;
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

/// Read the command and database path for a test file.
fn read_test_command(test_path: &str) -> Result<(String, String)> {
    if test_path.ends_with(&format!(".{}", RGT_EXTENSION)) {
        let spec = rgt::parse_rgt(test_path)?;
        let tdb_path = rgt_util::tdb_path_for_rgt(test_path);
        Ok((spec.command, tdb_path))
    } else {
        let original = reg_rs_store::db::read_original_results(test_path)?;
        Ok((original.command, test_path.to_string()))
    }
}

/// Format a test path as a display name (file stem only).
fn format_test_name(test_path: &str) -> String {
    std::path::Path::new(test_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

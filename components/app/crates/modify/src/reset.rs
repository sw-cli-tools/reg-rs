use reg_rs_config::config::Config;
use reg_rs_store::db;
use reg_rs_store::db_ops;
use reg_rs_store_rgt::rgt_util;
use reg_rs_types::constants::RGT_EXTENSION;
use reg_rs_types::error::Result;

/// Reset tests to clear latest results
pub fn reset(config: &Config) -> Result<()> {
    log::info!("command/reset");
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
    let mut reset_count = 0;
    for test_path in &tests.found {
        let is_rgt = test_path.ends_with(&format!(".{RGT_EXTENSION}"));
        let tdb_path = if is_rgt {
            rgt_util::tdb_path_for_rgt(test_path)
        } else {
            test_path.to_string()
        };
        db::reset_latest_results(&tdb_path)?;
        db_ops::reset_differences(&tdb_path)?;
        reset_count += 1;
        let name = format_test_name(test_path);
        eprintln!("reset: {name}");
    }
    eprintln!("{reset_count} test(s) reset");
    log::info!("command/reset done");
    Ok(())
}

/// Format a test path as a display name (file stem only).
fn format_test_name(test_path: &str) -> String {
    std::path::Path::new(test_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

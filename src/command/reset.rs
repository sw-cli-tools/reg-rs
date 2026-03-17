use crate::config;
use crate::db;
use crate::finder;
use crate::rgt;

use super::utils::format_test_name;

/// Reset tests to clear latest results
pub fn reset(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/reset");
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
    let mut reset_count = 0;
    for test_path in &tests.found {
        let is_rgt = test_path.ends_with(&format!(".{}", rgt::RGT_EXTENSION));
        let tdb_path = if is_rgt {
            rgt::tdb_path_for_rgt(test_path)
        } else {
            test_path.to_string()
        };
        db::reset_latest_results(&tdb_path)?;
        db::reset_differences(&tdb_path)?;
        reset_count += 1;
        let name = format_test_name(test_path);
        eprintln!("reset: {}", name);
    }
    eprintln!("{} test(s) reset", reset_count);
    log::info!("command/reset done");
    Ok(())
}

use reg_rs_config::config::Config;
use reg_rs_store::db_ops;
use reg_rs_store_rgt::rgt_util;
use reg_rs_types::error::Result;

/// Report on test results
pub fn report(config: &Config) -> Result<u32> {
    log::info!("command/report");
    let pattern = config.extract_pattern().to_string();
    let tests = reg_rs_discover::finder::discover(pattern.clone())?;
    if tests.found.is_empty() {
        eprintln!(
            "warning: no tests matched pattern '{}' in {}",
            pattern,
            tests.data_dir.display()
        );
        return Ok(0);
    }
    if config.is_quiet() {
        let mut fail_count = 0u32;
        for test_path in &tests.found {
            let db_path = rgt_util::db_path(test_path);
            let latest_count = db_ops::count_latest_results(&db_path)?;
            if latest_count > 0 && db_ops::count_differences(&db_path)? > 0 {
                fail_count += 1;
            }
        }
        return Ok(fail_count);
    }
    let fail_count =
        reg_rs_report::output::generate_reports(&pattern, &tests.found, config.verbosity_level())?;
    log::info!("command/report done");
    Ok(fail_count)
}

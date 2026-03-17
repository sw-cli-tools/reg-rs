use crate::config;
use crate::db;
use crate::finder;
use crate::reporters;

/// Report on test results
pub fn report(config: &config::Config) -> crate::error::Result<u32> {
    log::info!("command/report");
    if config.is_quiet() {
        let pattern = config.extract_pattern().to_string();
        let tests = finder::discover(pattern)?;
        let mut fail_count = 0u32;
        for test_path in &tests.found {
            let db = crate::db_path(test_path);
            let latest_count = db::count_latest_results(&db)?;
            if latest_count > 0 && db::count_differences(&db)? > 0 {
                fail_count += 1;
            }
        }
        return Ok(fail_count);
    }
    let fail_count = reporters::generate_reports(config)?;
    log::info!("command/report done");
    Ok(fail_count)
}

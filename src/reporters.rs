use crate::config;
use crate::db;
use crate::finder;

/// test reporting details
pub mod details;
/// test differences
pub mod differences;
/// test failures
pub mod failures;
/// test passes
pub mod passes;
/// test summary
pub mod summary;

/// Generate reports and return the number of failed tests.
pub(crate) fn generate_reports(config: &config::Config) -> crate::error::Result<u32> {
    log::info!("reporters/generate_reports");
    log::debug!("generate_reports");
    let pattern = config.extract_pattern().to_string();
    let test_names = finder::discover(pattern.clone())?;
    if test_names.found.is_empty() {
        eprintln!(
            "warning: no tests matched pattern '{}' in {}",
            pattern,
            test_names.data_dir.display()
        );
        return Ok(0);
    }
    let total_count = test_names.found.len() as u32;
    let mut failed_test_names = vec![];
    let mut passed_test_names = vec![];
    let mut not_yet_run_test_names = vec![];
    for test_name in &test_names.found {
        let db_path = crate::db_path(test_name);
        let latest_results_row_count = db::count_latest_results(&db_path)?;
        if latest_results_row_count == 0 {
            not_yet_run_test_names.push(test_name.to_string());
        } else {
            let difference_count = db::count_differences(&db_path)?;
            log::debug!("difference_count: {}", difference_count);
            if difference_count > 0 {
                failed_test_names.push(test_name.to_string());
            } else {
                passed_test_names.push(test_name.to_string());
            }
        }
    }
    let fail_count = failed_test_names.len() as u32;
    let not_yet_run_count = not_yet_run_test_names.len() as u32;
    let pass_count = passed_test_names.len() as u32;
    summary::show_summary(&summary::SummaryReportContext::new(
        fail_count,
        not_yet_run_count,
        pass_count,
        config.extract_pattern(),
        total_count,
    ))?;
    if config.verbosity_level() > 0 {
        let no_failed_tests = failed_test_names.is_empty();
        let no_not_yet_run_tests = not_yet_run_test_names.is_empty();
        let no_passed_tests = passed_test_names.is_empty();
        details::show_details(
            &details::DetailsReportContext::new(
                failed_test_names,
                no_failed_tests,
                no_not_yet_run_tests,
                no_passed_tests,
                not_yet_run_test_names,
                passed_test_names,
            ),
            config.verbosity_level(),
        )?;
    }
    Ok(fail_count)
}

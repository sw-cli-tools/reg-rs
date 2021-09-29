use crate::config;
use crate::db;
use crate::finder;

pub mod details;
pub mod differences;
pub mod failures;
pub mod passes;
pub mod summary;

pub fn generate_reports(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("reporters/generate_reports");
    md!("generate_report");
    let test_names = finder::discover(config.extract_pattern().to_string())?;
    let total_count = test_names.found.len() as u32;
    let mut failed_test_names = vec![];
    let mut passed_test_names = vec![];
    let mut not_yet_run_test_names = vec![];
    for test_name in &test_names.found {
        let latest_results_row_count = db::count_latest_results(test_name)?;
        if latest_results_row_count == 0 {
            not_yet_run_test_names.push(test_name.to_string());
        } else {
            let difference_count = db::count_differences(test_name)?;
            md!(difference_count);
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
        &config.extract_pattern().to_string(),
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
    Ok(())
}

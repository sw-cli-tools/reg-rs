use crate::config;
use crate::db;
use crate::finder;

pub mod details;
pub mod differences;
pub mod failures;
pub mod summary;

pub fn generate_reports(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    md!("generate_report");
    let test_names = finder::discover(&config)?;
    let total_count = *&test_names.found.len() as u32;
    let mut failed_test_names = vec![];
    let mut passed_test_names = vec![];
    let mut not_yet_run_test_names = vec![];
    for test_name in &test_names.found {
        let latest_results_table_count = db::latest_results_table_count(&test_name)?;
        if latest_results_table_count == 0 {
            not_yet_run_test_names.push(format!("{}", &test_name));
        } else {
            let difference_count = db::count_differences(&test_name)?;
            md!(difference_count);
            if difference_count > 0 {
                failed_test_names.push(format!("{}", &test_name));
            } else {
                passed_test_names.push(format!("{}", &test_name));
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
    let no_failed_tests = 0 == *&failed_test_names.len();
    let no_not_yet_run_tests = 0 == *&not_yet_run_test_names.len();
    let no_passed_tests = 0 == *&passed_test_names.len();
    details::show_details(&details::DetailsReportContext::new(
        failed_test_names,
        no_failed_tests,
        no_not_yet_run_tests,
        no_passed_tests,
        not_yet_run_test_names,
        passed_test_names,
    ))?;
    Ok(())
}

use reg_rs_store::db_ops;
use reg_rs_store_rgt::rgt as rgt_store;

use crate::details;
use crate::summary;

/// Categorized test names split into failed, passed, and not-yet-run buckets
struct CategorizedTests {
    failed: Vec<String>,
    passed: Vec<String>,
    not_yet_run: Vec<String>,
}

/// Sort tests into failed, passed, and not-yet-run categories
fn categorize_tests(found_tests: &[String]) -> reg_rs_types::error::Result<CategorizedTests> {
    let mut failed = vec![];
    let mut passed = vec![];
    let mut not_yet_run = vec![];
    for test_name in found_tests {
        let db_path = rgt_store::db_path(test_name);
        let latest_results_row_count = db_ops::count_latest_results(&db_path)?;
        if latest_results_row_count == 0 {
            not_yet_run.push(test_name.to_string());
        } else {
            let difference_count = db_ops::count_differences(&db_path)?;
            log::debug!("difference_count: {difference_count}");
            if difference_count > 0 {
                failed.push(test_name.to_string());
            } else {
                passed.push(test_name.to_string());
            }
        }
    }
    Ok(CategorizedTests {
        failed,
        passed,
        not_yet_run,
    })
}

/// Generate reports and return the number of failed tests.
pub fn generate_reports(
    pattern: &str,
    found_tests: &[String],
    verbosity_level: u8,
) -> reg_rs_types::error::Result<u32> {
    log::info!("reporters/generate_reports");
    let total_count = found_tests.len() as u32;
    let cats = categorize_tests(found_tests)?;
    let fail_count = cats.failed.len() as u32;
    let not_yet_run_count = cats.not_yet_run.len() as u32;
    let pass_count = cats.passed.len() as u32;
    summary::show_summary(&summary::SummaryReportContext::new(
        fail_count,
        not_yet_run_count,
        pass_count,
        pattern,
        total_count,
    ))?;
    if verbosity_level > 0 {
        let no_failed_tests = cats.failed.is_empty();
        let no_not_yet_run_tests = cats.not_yet_run.is_empty();
        let no_passed_tests = cats.passed.is_empty();
        details::show_details(
            &details::DetailsReportContext::new(
                cats.failed,
                no_failed_tests,
                no_not_yet_run_tests,
                no_passed_tests,
                cats.not_yet_run,
                cats.passed,
            ),
            verbosity_level,
        )?;
    }
    Ok(fail_count)
}

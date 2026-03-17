/// Test run state collection from database
use reg_rs_store::db;
use reg_rs_store::db_ops;
use reg_rs_store_rgt::rgt as rgt_store;

use crate::diff_format;
use crate::templates::TestDetails;

/// Collect test run data from a list of test paths
pub fn collect_test_runs(test_paths: &[String]) -> reg_rs_types::error::Result<Vec<TestDetails>> {
    let mut test_runs = vec![];
    for test_name in test_paths {
        let db_path = rgt_store::db_path(test_name);
        let original_result = db::read_original_results(&db_path)?;
        let (last_ran, diffs) = collect_latest_result(&db_path)?;
        test_runs.push(TestDetails {
            created: original_result.time_created,
            diffs,
            name: test_name.to_string(),
            last_ran,
        });
    }
    Ok(test_runs)
}

/// Collect latest result data from database
fn collect_latest_result(
    db_path: &str,
) -> reg_rs_types::error::Result<(Option<String>, Option<Vec<String>>)> {
    let latest_results_row_count = db_ops::count_latest_results(db_path)?;
    if latest_results_row_count == 0 {
        Ok((None, None))
    } else {
        let latest_result = db::read_latest_results(db_path)?;
        let difference_count = db_ops::count_differences(db_path)?;
        let diffs = if difference_count > 0 {
            Some(get_diffs(db_path)?)
        } else {
            None
        };
        Ok((Some(latest_result.time_created), diffs))
    }
}

/// Get diff HTML strings from database
fn get_diffs(test_name: &str) -> reg_rs_types::error::Result<Vec<String>> {
    log::info!("server/get_diffs test_name {}", &test_name);
    let differences = db_ops::read_differences(test_name)?;
    Ok(diff_format::format_diffs_html(&differences))
}

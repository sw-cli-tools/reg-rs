/// Test run state management
use crate::db;
use crate::finder;
use crate::status::AppState;

/// Update shared state with test run data
pub fn set_test_runs(app_state: AppState) -> crate::error::Result<()> {
    log::info!("server/set_test_runs - acquiring lock");
    let mut state_data = app_state.state_data.lock().map_err(|e| {
        crate::error::RegError::MutexPoisoned(format!("state_data lock failed: {}", e))
    })?;
    log::info!(
        "server/set_test_runs - lock acquired, pattern: {}",
        &state_data.pattern
    );

    let test_runs = collect_test_runs(&state_data)?;
    state_data.runs = test_runs;
    state_data.state_updated = crate::time::now();

    log::info!(
        "server/set_test_runs - completed, {} tests loaded, releasing lock",
        state_data.runs.len()
    );
    Ok(())
}

/// Collect test run data from database
fn collect_test_runs(
    state_data: &crate::status::StateData,
) -> crate::error::Result<Vec<crate::status::TestDetails>> {
    let mut test_runs = vec![];
    let test_names = finder::discover(state_data.pattern.to_string())?;
    for test_name in &test_names.found {
        let db = crate::db_path(test_name);
        let original_result = db::read_original_results(&db)?;
        let (last_ran, diffs) = collect_latest_result(&db)?;
        test_runs.push(crate::status::TestDetails {
            created: original_result.time_created,
            diffs,
            name: test_name.to_string(),
            last_ran,
        });
    }
    Ok(test_runs)
}

/// Collect latest result data from database
fn collect_latest_result(db: &str) -> crate::error::Result<(Option<String>, Option<Vec<String>>)> {
    let latest_results_row_count = db::count_latest_results(db)?;
    if latest_results_row_count == 0 {
        Ok((None, None))
    } else {
        let latest_result = db::read_latest_results(db)?;
        let difference_count = db::count_differences(db)?;
        let diffs = if difference_count > 0 {
            Some(get_diffs(db)?)
        } else {
            None
        };
        Ok((Some(latest_result.time_created), diffs))
    }
}

/// Get diff HTML strings from database
fn get_diffs(test_name: &str) -> crate::error::Result<Vec<String>> {
    use super::diff_formatter;

    log::info!("server/get_diffs test_name {}", &test_name);
    let differences = db::read_differences(test_name)?;
    Ok(diff_formatter::format_diffs_html(&differences))
}

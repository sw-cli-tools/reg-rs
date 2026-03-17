/// Status view building utilities
use super::state::StateData;
use super::templates::{StatusCounts, StatusFlags, StatusViewContext, categorize_runs, render};

/// Build status view HTML from current state data
pub fn build_status_view(state_data: &StateData) -> crate::error::Result<String> {
    let (failed_test_names, passed_test_names, not_yet_run_test_names) =
        categorize_runs(&state_data.runs);

    let status_counts = StatusCounts {
        fail_count: format!(" {:05}", failed_test_names.len()),
        not_run_count: format!(" {:05}", not_yet_run_test_names.len()),
        pass_count: format!(" {:05}", passed_test_names.len()),
        test_count: format!(" {:05}", state_data.runs.len()),
    };
    let status_flags = StatusFlags {
        no_failed_tests: failed_test_names.is_empty(),
        no_not_yet_run_tests: not_yet_run_test_names.is_empty(),
        no_passed_tests: passed_test_names.is_empty(),
    };

    render(&StatusViewContext::new(
        state_data.server_started.clone(),
        state_data.state_updated.clone(),
        status_counts,
        status_flags,
        state_data.pattern.to_string(),
        &state_data.runs,
    ))
}

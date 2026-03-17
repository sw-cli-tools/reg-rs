use super::super::state::TestDetails;

/// Categorize test runs into failed, passed, and not-yet-run lists
pub fn categorize_runs(runs: &[TestDetails]) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut failed = vec![];
    let mut not_yet_run = vec![];
    let mut passed = vec![];
    for run in runs {
        if run.last_ran.is_none() {
            not_yet_run.push(run.name.clone());
        } else if run.diffs.is_none() {
            passed.push(run.name.clone());
        } else {
            failed.push(run.name.clone());
        }
    }
    (failed, passed, not_yet_run)
}

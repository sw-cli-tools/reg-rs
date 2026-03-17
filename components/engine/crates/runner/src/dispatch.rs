use reg_rs_store::db;
use reg_rs_store_rgt::rgt;
use reg_rs_types::types::TestResults;

use crate::diff;
use crate::runner;

/// Run a single test and process differences.
///
/// Supports both `.rgt` and `.tdb` test sources.
pub fn run_and_diff(test: &str, dry_run: bool) -> reg_rs_types::error::Result<()> {
    if rgt::is_rgt_path(test) {
        run_and_diff_rgt(test, dry_run)
    } else {
        run_and_diff_tdb(test, dry_run)
    }
}

/// Run a .tdb test (legacy path)
pub fn run_and_diff_tdb(test: &str, dry_run: bool) -> reg_rs_types::error::Result<()> {
    let prior_test_result = db::read_original_results(test)?;
    let timeout_secs = reg_rs_store::db_ops::read_metadata(test, "timeout")?
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(300);
    let maybe_regression =
        runner::run_one_timeout(test, &prior_test_result.command, dry_run, timeout_secs)?;
    if let Some(latest_test_result) = maybe_regression {
        diff::process_differences(test, &prior_test_result, &latest_test_result)?;
        db::replace_latest_results(test, &latest_test_result)?;
    }
    Ok(())
}

/// Run an .rgt test: read spec from TOML, baselines from .out/.err, store in .tdb cache
pub fn run_and_diff_rgt(rgt_path: &str, dry_run: bool) -> reg_rs_types::error::Result<()> {
    let spec = rgt::parse_rgt(rgt_path)?;
    let timeout_secs = spec.timeout.unwrap_or(300);
    let tdb_path = rgt::tdb_path_for_rgt(rgt_path);

    let maybe_result = runner::run_one_timeout(rgt_path, &spec.command, dry_run, timeout_secs)?;
    if let Some(latest_test_result) = maybe_result {
        let baseline_stdout = rgt::read_baseline_stdout(rgt_path)?;
        let baseline_stderr = rgt::read_baseline_stderr(rgt_path)?;
        let prior_test_result = TestResults {
            name: rgt_path.to_string(),
            command: spec.command.clone(),
            time_created: String::new(),
            exit_code: spec.exit_code.unwrap_or(latest_test_result.exit_code),
            stderr: baseline_stderr,
            stdout: baseline_stdout,
        };

        diff::process_differences_with_settings(
            &tdb_path,
            &prior_test_result,
            &latest_test_result,
            spec.preprocess.as_deref(),
            spec.diff_mode.as_deref(),
        )?;
        db::replace_latest_results(&tdb_path, &latest_test_result)?;
    }
    Ok(())
}

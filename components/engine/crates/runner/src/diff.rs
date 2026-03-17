use text_diff::{Difference, diff};

use reg_rs_exec::preprocess;
use reg_rs_store::db_ops;
use reg_rs_types::error::Result;
use reg_rs_types::normalize;
use reg_rs_types::types::{RegressionType, TestResults};

use crate::diff_store;

/// Determine test result differences
///
/// Returns `Some` if there are differences, `None` if the strings are identical.
pub fn get_differences(older: &str, newer: &str) -> Option<Vec<Difference>> {
    log::info!("diff/get_differences");
    let differences = diff(older, newer, "\n");
    log::debug!("get_differences: {:?}", &differences);
    if differences.0 > 0 {
        Some(differences.1)
    } else {
        None
    }
}

/// Prepare test results for diffing by applying preprocess and normalization.
fn prepare_for_diff(
    test_result: &TestResults,
    preprocess_cmd: Option<&str>,
    diff_mode: &normalize::DiffMode,
) -> Result<TestResults> {
    let stdout = preprocess::apply(&test_result.stdout, preprocess_cmd)?;
    let stderr = preprocess::apply(&test_result.stderr, preprocess_cmd)?;
    let stdout = normalize::apply(&stdout, diff_mode)?;
    Ok(TestResults {
        name: test_result.name.clone(),
        command: test_result.command.clone(),
        time_created: test_result.time_created.clone(),
        exit_code: test_result.exit_code,
        stdout,
        stderr,
    })
}

/// Store test result differences (reads settings from DB metadata)
pub fn process_differences(
    db_name: &str,
    prior_test_result: &TestResults,
    latest_test_result: &TestResults,
) -> Result<()> {
    log::info!("diff/process_differences {}", &db_name);

    let pp = db_ops::read_metadata(db_name, reg_rs_types::constants::PREPROCESS_KEY)?;
    let pp_ref = pp.as_deref();

    let diff_mode_str = db_ops::read_metadata(db_name, reg_rs_types::constants::DIFF_MODE_KEY)?;
    let diff_mode = diff_mode_str
        .as_deref()
        .map(|s| s.parse::<normalize::DiffMode>())
        .transpose()?
        .unwrap_or_default();

    store_differences(
        db_name,
        prior_test_result,
        latest_test_result,
        pp_ref,
        &diff_mode,
    )
}

/// Store test result differences with explicit preprocess and diff_mode settings.
pub fn process_differences_with_settings(
    db_name: &str,
    prior_test_result: &TestResults,
    latest_test_result: &TestResults,
    preprocess_cmd: Option<&str>,
    diff_mode_str: Option<&str>,
) -> Result<()> {
    log::info!("diff/process_differences_with_settings {}", &db_name);

    let diff_mode = diff_mode_str
        .map(|s| s.parse::<normalize::DiffMode>())
        .transpose()?
        .unwrap_or_default();

    store_differences(
        db_name,
        prior_test_result,
        latest_test_result,
        preprocess_cmd,
        &diff_mode,
    )
}

/// Common implementation for storing differences.
fn store_differences(
    db_name: &str,
    prior_test_result: &TestResults,
    latest_test_result: &TestResults,
    pp_ref: Option<&str>,
    diff_mode: &normalize::DiffMode,
) -> Result<()> {
    let prior = prepare_for_diff(prior_test_result, pp_ref, diff_mode)?;
    let latest = prepare_for_diff(latest_test_result, pp_ref, diff_mode)?;

    db_ops::clear_differences(db_name)?;
    diff_store::maybe_store_exit_code_differences(db_name, &prior, &latest)?;
    diff_store::maybe_store_stream_differences(
        db_name,
        &prior.stderr,
        &latest.stderr,
        RegressionType::StderrAdd,
        RegressionType::StderrRemove,
        RegressionType::StderrSame,
    )?;
    diff_store::maybe_store_stream_differences(
        db_name,
        &prior.stdout,
        &latest.stdout,
        RegressionType::StdoutAdd,
        RegressionType::StdoutRemove,
        RegressionType::StdoutSame,
    )?;
    Ok(())
}

use text_diff::Difference;

use reg_rs_store::db_ops;
use reg_rs_types::error::Result;
use reg_rs_types::types::{RegressionType, TestResults};

use crate::diff;

/// Store exit code differences if they differ
pub fn maybe_store_exit_code_differences(
    db_name: &str,
    prior: &TestResults,
    latest: &TestResults,
) -> Result<()> {
    log::info!("diff/maybe_store_exit_code_differences {}", &db_name);
    if prior.exit_code != latest.exit_code {
        log::debug!(
            "exit code diff: {} vs {}",
            prior.exit_code,
            latest.exit_code
        );
        db_ops::store_difference(
            db_name,
            RegressionType::ExpectedCode,
            &prior.exit_code.to_string(),
        )?;
        db_ops::store_difference(
            db_name,
            RegressionType::ActualCode,
            &latest.exit_code.to_string(),
        )?;
    }
    Ok(())
}

/// Store stream (stdout or stderr) differences using parameterized types.
///
/// This collapses the previous `maybe_store_stderr_differences` and
/// `maybe_store_stdout_differences` into a single function.
pub fn maybe_store_stream_differences(
    db_name: &str,
    prior_stream: &str,
    latest_stream: &str,
    add_type: RegressionType,
    remove_type: RegressionType,
    same_type: RegressionType,
) -> Result<()> {
    if let Some(differences) = diff::get_differences(prior_stream, latest_stream) {
        for difference in differences.iter() {
            match difference {
                Difference::Add(add) => {
                    db_ops::store_difference(db_name, add_type, add)?;
                }
                Difference::Rem(rem) => {
                    db_ops::store_difference(db_name, remove_type, rem)?;
                }
                Difference::Same(same) => {
                    db_ops::store_difference(db_name, same_type, same)?;
                }
            }
        }
    }
    Ok(())
}

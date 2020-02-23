use log;
use text_diff::{diff, Difference};

use crate::db;
use crate::runner;

pub enum RegressionType {
    ActualCode = 1,
    ExpectedCode,
    StderrAdd,
    StderrRemove,
    StderrSame,
    StdoutAdd,
    StdoutRemove,
    StdoutSame,
}

pub fn get_differences(older: &str, newer: &str) -> Option<Vec<Difference>> {
    log::info!("diff/get_differences");
    let differences = diff(older, newer, "\n");
    md!("*** get_differences ***");
    md!(&differences);
    if differences.0 > 0 {
        Some(differences.1)
    } else {
        None
    }
}

pub fn process_differences(
    db_name: &str,
    prior_test_result: &runner::TestResults,
    latest_test_result: &runner::TestResults,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("diff/process_differences {}", &db_name);
    db::reset_differences(&db_name)?;
    maybe_store_exit_code_differences(&db_name, &prior_test_result, &latest_test_result)?;
    maybe_store_stderr_differences(&db_name, &prior_test_result, &latest_test_result)?;
    maybe_store_stdout_differences(&db_name, &prior_test_result, &latest_test_result)?;
    Ok(())
}

fn maybe_store_exit_code_differences(
    db_name: &str,
    prior_test_result: &runner::TestResults,
    latest_test_result: &runner::TestResults,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("diff/maybe_store_exit_code_differences {}", &db_name);
    if prior_test_result.exit_code != latest_test_result.exit_code {
        md!((prior_test_result.exit_code, latest_test_result.exit_code));
        db::store_difference(
            &db_name,
            RegressionType::ExpectedCode,
            &prior_test_result.exit_code.to_string(),
        )?;
        db::store_difference(
            &db_name,
            RegressionType::ActualCode,
            &latest_test_result.exit_code.to_string(),
        )?;
    }
    Ok(())
}

fn maybe_store_stderr_differences(
    db_name: &str,
    prior_test_result: &runner::TestResults,
    latest_test_result: &runner::TestResults,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("diff/maybe_store_stderr_differences {}", &db_name);
    if let Some(differences) =
        get_differences(&prior_test_result.stderr, &latest_test_result.stderr)
    {
        md!("*** stderr differences ***");
        for difference in differences.iter() {
            md!(difference);
            match difference {
                Difference::Add(add) => {
                    db::store_difference(&db_name, RegressionType::StderrAdd, add)?;
                }
                Difference::Rem(rem) => {
                    db::store_difference(&db_name, RegressionType::StderrRemove, rem)?;
                }
                Difference::Same(same) => {
                    db::store_difference(&db_name, RegressionType::StderrSame, same)?;
                }
            }
        }
    }
    Ok(())
}

fn maybe_store_stdout_differences(
    db_name: &str,
    prior_test_result: &runner::TestResults,
    latest_test_result: &runner::TestResults,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("diff/maybe_store_stdout_differences {}", &db_name);
    if let Some(differences) =
        get_differences(&prior_test_result.stdout, &latest_test_result.stdout)
    {
        md!("*** stdout differences ***");
        for difference in differences.iter() {
            md!(difference);
            match difference {
                Difference::Add(add) => {
                    db::store_difference(&db_name, RegressionType::StdoutAdd, add)?;
                }
                Difference::Rem(rem) => {
                    db::store_difference(&db_name, RegressionType::StdoutRemove, rem)?;
                }
                Difference::Same(same) => {
                    db::store_difference(&db_name, RegressionType::StdoutSame, same)?;
                }
            }
        }
    }
    Ok(())
}

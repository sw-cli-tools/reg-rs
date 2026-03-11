use text_diff::{Difference, diff};

use crate::db;
use crate::error::Result;
use crate::runner;

/// test result regression types
pub enum RegressionType {
    /// The latest test result exit code
    ActualCode = 1,
    /// The reference test result exit code
    ExpectedCode,
    /// The latest test result stderr additions
    StderrAdd,
    /// The latest test result stderr removals
    StderrRemove,
    /// The latest test result stderr matches
    StderrSame,
    /// The latest test result stdout additions
    StdoutAdd,
    /// The latest test result stdout removals
    StdoutRemove,
    /// The latest test result stdout matches
    StdoutSame,
}

impl RegressionType {
    /// Convert a stored type code string to a human-readable label for display.
    /// Returns None for "same" types (5, 8) and unknown codes.
    pub fn display_label(type_code: &str) -> Option<&'static str> {
        let code: u8 = type_code.parse().ok()?;
        match code {
            x if x == Self::ActualCode as u8 => Some("Actual exit code"),
            x if x == Self::ExpectedCode as u8 => Some("Expected exit code"),
            x if x == Self::StderrAdd as u8 => Some("stderr add"),
            x if x == Self::StderrRemove as u8 => Some("stderr remove"),
            x if x == Self::StdoutAdd as u8 => Some("stdout add"),
            x if x == Self::StdoutRemove as u8 => Some("stdout remove"),
            _ => None,
        }
    }

    /// Check if a type code represents a "has differences" type for counting.
    pub fn has_differences(type_code: &str) -> bool {
        let Ok(code) = type_code.parse::<u8>() else {
            return false;
        };
        matches!(
            code,
            x if x == Self::ActualCode as u8
                || x == Self::StderrAdd as u8
                || x == Self::StderrRemove as u8
                || x == Self::StdoutAdd as u8
                || x == Self::StdoutRemove as u8
        )
    }
}

/// determine test result differences
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

/// store test result differences
pub fn process_differences(
    db_name: &str,
    prior_test_result: &runner::TestResults,
    latest_test_result: &runner::TestResults,
) -> Result<()> {
    log::info!("diff/process_differences {}", &db_name);
    db::clear_differences(db_name)?;
    maybe_store_exit_code_differences(db_name, prior_test_result, latest_test_result)?;
    maybe_store_stderr_differences(db_name, prior_test_result, latest_test_result)?;
    maybe_store_stdout_differences(db_name, prior_test_result, latest_test_result)?;
    Ok(())
}

/// store test result exit code differences
fn maybe_store_exit_code_differences(
    db_name: &str,
    prior_test_result: &runner::TestResults,
    latest_test_result: &runner::TestResults,
) -> Result<()> {
    log::info!("diff/maybe_store_exit_code_differences {}", &db_name);
    if prior_test_result.exit_code != latest_test_result.exit_code {
        log::debug!(
            "exit code diff: {} vs {}",
            prior_test_result.exit_code,
            latest_test_result.exit_code
        );
        db::store_difference(
            db_name,
            RegressionType::ExpectedCode,
            &prior_test_result.exit_code.to_string(),
        )?;
        db::store_difference(
            db_name,
            RegressionType::ActualCode,
            &latest_test_result.exit_code.to_string(),
        )?;
    }
    Ok(())
}

/// store test result standard error differences
fn maybe_store_stderr_differences(
    db_name: &str,
    prior_test_result: &runner::TestResults,
    latest_test_result: &runner::TestResults,
) -> Result<()> {
    log::info!("diff/maybe_store_stderr_differences {}", &db_name);
    if let Some(differences) =
        get_differences(&prior_test_result.stderr, &latest_test_result.stderr)
    {
        log::debug!("stderr differences");
        for difference in differences.iter() {
            log::debug!("stderr diff: {:?}", difference);
            match difference {
                Difference::Add(add) => {
                    db::store_difference(db_name, RegressionType::StderrAdd, add)?;
                }
                Difference::Rem(rem) => {
                    db::store_difference(db_name, RegressionType::StderrRemove, rem)?;
                }
                Difference::Same(same) => {
                    db::store_difference(db_name, RegressionType::StderrSame, same)?;
                }
            }
        }
    }
    Ok(())
}

/// store test result standard output differences
fn maybe_store_stdout_differences(
    db_name: &str,
    prior_test_result: &runner::TestResults,
    latest_test_result: &runner::TestResults,
) -> Result<()> {
    log::info!("diff/maybe_store_stdout_differences {}", &db_name);
    if let Some(differences) =
        get_differences(&prior_test_result.stdout, &latest_test_result.stdout)
    {
        log::debug!("stdout differences");
        for difference in differences.iter() {
            log::debug!("stdout diff: {:?}", difference);
            match difference {
                Difference::Add(add) => {
                    db::store_difference(db_name, RegressionType::StdoutAdd, add)?;
                }
                Difference::Rem(rem) => {
                    db::store_difference(db_name, RegressionType::StdoutRemove, rem)?;
                }
                Difference::Same(same) => {
                    db::store_difference(db_name, RegressionType::StdoutSame, same)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_differences_identical() {
        assert!(get_differences("hello", "hello").is_none());
    }

    #[test]
    fn test_get_differences_different() {
        let diffs = get_differences("hello", "world");
        assert!(diffs.is_some());
        let diffs = diffs.unwrap();
        assert!(!diffs.is_empty());
    }

    #[test]
    fn test_get_differences_empty_strings() {
        assert!(get_differences("", "").is_none());
    }

    #[test]
    fn test_get_differences_added_content() {
        let diffs = get_differences("", "new content").unwrap();
        assert!(diffs.iter().any(|d| matches!(d, Difference::Add(_))));
    }

    #[test]
    fn test_get_differences_removed_content() {
        let diffs = get_differences("old content", "").unwrap();
        assert!(diffs.iter().any(|d| matches!(d, Difference::Rem(_))));
    }

    #[test]
    fn test_display_label_known_types() {
        assert_eq!(RegressionType::display_label("1"), Some("Actual exit code"));
        assert_eq!(
            RegressionType::display_label("2"),
            Some("Expected exit code")
        );
        assert_eq!(RegressionType::display_label("3"), Some("stderr add"));
        assert_eq!(RegressionType::display_label("4"), Some("stderr remove"));
        assert_eq!(RegressionType::display_label("6"), Some("stdout add"));
        assert_eq!(RegressionType::display_label("7"), Some("stdout remove"));
    }

    #[test]
    fn test_display_label_same_types_return_none() {
        assert_eq!(RegressionType::display_label("5"), None);
        assert_eq!(RegressionType::display_label("8"), None);
    }

    #[test]
    fn test_display_label_invalid() {
        assert_eq!(RegressionType::display_label("0"), None);
        assert_eq!(RegressionType::display_label("99"), None);
        assert_eq!(RegressionType::display_label("abc"), None);
    }

    #[test]
    fn test_has_differences() {
        assert!(RegressionType::has_differences("1")); // ActualCode
        assert!(!RegressionType::has_differences("2")); // ExpectedCode
        assert!(RegressionType::has_differences("3")); // StderrAdd
        assert!(RegressionType::has_differences("4")); // StderrRemove
        assert!(!RegressionType::has_differences("5")); // StderrSame
        assert!(RegressionType::has_differences("6")); // StdoutAdd
        assert!(RegressionType::has_differences("7")); // StdoutRemove
        assert!(!RegressionType::has_differences("8")); // StdoutSame
        assert!(!RegressionType::has_differences("abc"));
    }
}

use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::db;
use crate::diff;
use crate::reporters::differences;
use crate::reporters::failures;
use crate::reporters::passes;
use crate::templates::reports;
use crate::util::{fail_symbol, pass_symbol, warn, warn_symbol};

/// Data for details report template
#[derive(Serialize)]
pub struct DetailsReportContext {
    fail_symbol: String,
    failed_test_names: Vec<String>,
    no_failed_tests: bool,
    no_not_yet_run_tests: bool,
    no_passed_tests: bool,
    not_yet_run_test_names: Vec<String>,
    pass_symbol: String,
    passed_test_names: Vec<String>,
    warn_symbol: String,
}

impl DetailsReportContext {
    /// generate new data for a details report template
    pub fn new(
        failed_test_names: Vec<String>,
        no_failed_tests: bool,
        no_not_yet_run_tests: bool,
        no_passed_tests: bool,
        not_yet_run_test_names: Vec<String>,
        passed_test_names: Vec<String>,
    ) -> Self {
        DetailsReportContext {
            fail_symbol: fail_symbol(),
            failed_test_names,
            no_failed_tests,
            no_not_yet_run_tests,
            no_passed_tests,
            not_yet_run_test_names,
            pass_symbol: pass_symbol(),
            passed_test_names,
            warn_symbol: warn_symbol(),
        }
    }
}

/// Render details report to a string
pub fn render(details_report_context: &DetailsReportContext) -> crate::error::Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template("details_report_template", reports::DETAILS_REPORT_TEMPLATE)?;
    let rendered = tt.render("details_report_template", details_report_context)?;
    Ok(rendered)
}

/// show test result report details
pub fn show_details(
    details_report_context: &DetailsReportContext,
    verbosity_level: u8,
) -> crate::error::Result<()> {
    log::info!("details/show_details");
    let rendered = render(details_report_context)?;
    println!("{}", rendered);
    if verbosity_level > 1 {
        show_failures(details_report_context, verbosity_level)?;
    }
    if verbosity_level > 1 {
        show_passes(details_report_context, verbosity_level)?;
    }
    Ok(())
}

/// show test result failures
fn show_failures(
    details_report_context: &DetailsReportContext,
    verbosity_level: u8,
) -> crate::error::Result<()> {
    log::info!("details/show_failures");
    let failed_test_names = &details_report_context.failed_test_names;
    println!("Failures: (-vv)");
    if failed_test_names.is_empty() {
        println!("  (none)");
    }
    for test in failed_test_names {
        log::debug!("show_failures test: {:?}", &test);
        let original_result = db::read_original_results(test)?;
        log::debug!("show_failures original: {:?}", &original_result);
        let latest_result = db::read_latest_results(test)?;
        log::debug!("show_failures latest: {:?}", &latest_result);
        let differences = db::read_differences(test)?;
        log::debug!("show_failures differences: {:?}", &differences);
        let mut difference_types = vec![];
        let same_count =
            db::difference_count_by_type(test, diff::RegressionType::StderrSame as u8)?
                + db::difference_count_by_type(test, diff::RegressionType::StdoutSame as u8)?;
        let differences_count = differences.len() as u32 - same_count;
        if verbosity_level > 2 {
            if 0 < db::difference_count_by_type(test, diff::RegressionType::ActualCode as u8)? {
                difference_types.push("exit_code".to_string());
            }
            if 0 < db::difference_count_by_type(test, diff::RegressionType::StderrAdd as u8)?
                || 0 < db::difference_count_by_type(test, diff::RegressionType::StderrRemove as u8)?
            {
                difference_types.push("stderr".to_string());
            }
            if 0 < db::difference_count_by_type(test, diff::RegressionType::StdoutAdd as u8)?
                || 0 < db::difference_count_by_type(test, diff::RegressionType::StdoutRemove as u8)?
            {
                difference_types.push("stdout".to_string());
            }
        }
        failures::show_failure(&failures::FailuresReportContext::new(
            difference_types,
            differences_count,
            fail_symbol(),
            test.to_string(),
            original_result.time_created.to_string(),
            latest_result.time_created.to_string(),
        ))?;
        show_doc_metadata(test)?;
        if verbosity_level > 2 {
            let display_differences: Vec<_> = differences
                .iter()
                .filter_map(|difference| {
                    diff::RegressionType::display_label(&difference.0).map(|label| {
                        differences::DisplayDifference {
                            type_name: format!("{:022}", label),
                            chunk: difference.1.to_string(),
                        }
                    })
                })
                .collect();
            log::debug!(
                "show_failures display_differences: {:?}",
                &display_differences
            );
            differences::show_differences(&differences::DifferencesReportContext::new(
                display_differences,
                test.to_string(),
            ))?;
        }
    }
    Ok(())
}

/// Show documentation metadata for a test, if any exists.
fn show_doc_metadata(test: &str) -> crate::error::Result<()> {
    let desc = db::read_metadata(test, "desc")?;
    let expects = db::read_metadata(test, "expects")?;
    let flaky_note = db::read_metadata(test, "flaky_note")?;
    if desc.is_some() || expects.is_some() || flaky_note.is_some() {
        if let Some(d) = desc {
            println!("    desc:       {}", d);
        }
        if let Some(e) = expects {
            println!("    expects:    {}", e);
        }
        if let Some(f) = flaky_note {
            println!("    flaky_note: {}", f);
        }
    }
    Ok(())
}

/// show test result passes
fn show_passes(
    details_report_context: &DetailsReportContext,
    verbosity_level: u8,
) -> crate::error::Result<()> {
    log::info!("details/show_passes");
    let passed_test_names = &details_report_context.passed_test_names;
    println!("Passes:");
    if passed_test_names.is_empty() {
        println!("  (none)");
    }
    for test in passed_test_names {
        log::debug!("show_passes test: {:?}", &test);
        let original_result = db::read_original_results(test)?;
        log::debug!("show_passes original: {:?}", &original_result);
        let latest_result = db::read_latest_results(test)?;
        log::debug!("show_passes latest: {:?}", &latest_result);
        passes::show_passes(&passes::PassesReportContext::new(
            pass_symbol(),
            test.to_string(),
            original_result.time_created.to_string(),
            latest_result.time_created.to_string(),
        ))?;
    }

    if verbosity_level > 3 {
        println!(
            "{} verbosity level {} exceeds max",
            warn("*warning*"),
            verbosity_level
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_all_categories() {
        let ctx = DetailsReportContext::new(
            vec!["failed_test".to_string()],
            false,
            false,
            false,
            vec!["pending_test".to_string()],
            vec!["passed_test".to_string()],
        );
        let output = render(&ctx).unwrap();
        assert!(output.contains("failed_test"));
        assert!(output.contains("pending_test"));
        assert!(output.contains("passed_test"));
    }

    #[test]
    fn test_render_no_failures() {
        let ctx = DetailsReportContext::new(
            vec![],
            true,
            false,
            false,
            vec!["pending".to_string()],
            vec!["ok".to_string()],
        );
        let output = render(&ctx).unwrap();
        assert!(output.contains("No Failed Tests"));
    }

    #[test]
    fn test_render_no_passes() {
        let ctx = DetailsReportContext::new(
            vec!["broken".to_string()],
            false,
            true,
            true,
            vec![],
            vec![],
        );
        let output = render(&ctx).unwrap();
        assert!(output.contains("No Passed Tests"));
    }

    #[test]
    fn test_render_empty() {
        let ctx = DetailsReportContext::new(vec![], true, true, true, vec![], vec![]);
        let output = render(&ctx).unwrap();
        assert!(output.contains("No Failed Tests"));
        assert!(output.contains("No Passed Tests"));
    }
}

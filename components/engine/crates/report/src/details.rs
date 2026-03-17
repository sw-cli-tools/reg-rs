use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::details_failures;
use crate::details_passes;
use crate::format::{fail_symbol, pass_symbol, warn_symbol};

/// Details report template
const DETAILS_REPORT_TEMPLATE: &str ="
* Details * (-v){{ if no_failed_tests }}
No Failed Tests{{ else }}
{ fail_symbol } Failures: {{ for failed_test in failed_test_names }}{ failed_test }{{ if not @last }}, {{ endif }}{{ endfor }}{{ endif }}
{{- if no_not_yet_run_tests }}{{ else }}
{ warn_symbol } Not Yet Run: {{ for not_yet_run_test in not_yet_run_test_names }}{ not_yet_run_test }{{ if not @last }}, {{ endif }}{{ endfor }}{{ endif -}}
{{ if no_passed_tests }}
No Passed Tests{{ else }}
{ pass_symbol } Passed: {{ for passed_test in passed_test_names }}{ passed_test }{{ if not @last }}, {{ endif }}{{ endfor }}{{ endif }}
";

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
    /// Generate new data for a details report template
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

    /// Access the failed test names
    pub fn failed_test_names(&self) -> &[String] {
        &self.failed_test_names
    }

    /// Access the passed test names
    pub fn passed_test_names(&self) -> &[String] {
        &self.passed_test_names
    }
}

/// Render details report to a string
pub fn render(
    details_report_context: &DetailsReportContext,
) -> reg_rs_types::error::Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template("details_report_template", DETAILS_REPORT_TEMPLATE)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    let rendered = tt
        .render("details_report_template", details_report_context)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    Ok(rendered)
}

/// Show test result report details
pub fn show_details(
    details_report_context: &DetailsReportContext,
    verbosity_level: u8,
) -> reg_rs_types::error::Result<()> {
    log::info!("details/show_details");
    let rendered = render(details_report_context)?;
    println!("{}", rendered);
    if verbosity_level > 1 {
        details_failures::show_failures(details_report_context, verbosity_level)?;
    }
    if verbosity_level > 1 {
        details_passes::show_passes(details_report_context, verbosity_level)?;
    }
    Ok(())
}

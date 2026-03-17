use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::format::{fail, pass, warn};

/// Summary report template
const SUMMARY_REPORT_TEMPLATE: &str = "reg-rs Summary Report { report_date }
{ fail_count } failed
{ not_run_count } not yet run
{ pass_count } passed
 -----
{ test_count } matched pattern: { test_pattern }";

/// Data for test summary report template
#[derive(Serialize)]
pub struct SummaryReportContext {
    fail_count: String,
    not_run_count: String,
    pass_count: String,
    report_date: String,
    test_count: String,
    test_pattern: String,
}

impl SummaryReportContext {
    /// Build data for test summary report template
    pub fn new(
        fail_count: u32,
        not_yet_run_count: u32,
        pass_count: u32,
        pattern: &str,
        test_count: u32,
    ) -> Self {
        let date = chrono::Local::now();
        let report_date = format!("{}", date.format("%Y-%m-%dT%H:%M:%S"));
        SummaryReportContext {
            fail_count: maybe_color(fail_count > 0, &fail, fail_count),
            not_run_count: maybe_color(not_yet_run_count > 0, &warn, not_yet_run_count),
            pass_count: maybe_color(pass_count > 0, &pass, pass_count),
            report_date,
            test_count: maybe_color(test_count == 0, &warn, test_count),
            test_pattern: pattern.to_string(),
        }
    }
}

/// Conditionally color output
fn maybe_color(condition: bool, cb: &dyn Fn(&str) -> String, count: u32) -> String {
    let s = format!(" {:05}", count);
    if condition { cb(&s) } else { s }
}

/// Render summary report to a string
pub fn render(
    summary_report_context: &SummaryReportContext,
) -> reg_rs_types::error::Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template("summary_report_template", SUMMARY_REPORT_TEMPLATE)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    let rendered = tt
        .render("summary_report_template", summary_report_context)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    Ok(rendered)
}

/// Show summary template rendered output
pub fn show_summary(
    summary_report_context: &SummaryReportContext,
) -> reg_rs_types::error::Result<()> {
    log::info!("summary/show_summary");
    let rendered = render(summary_report_context)?;
    println!("{}", rendered);
    Ok(())
}

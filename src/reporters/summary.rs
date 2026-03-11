use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::templates::reports;
use crate::time;
use crate::util::{fail, pass, warn};

/// data for test summary report template
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
    /// build data for test summary report template
    pub fn new(
        fail_count: u32,
        not_yet_run_count: u32,
        pass_count: u32,
        pattern: &str,
        test_count: u32,
    ) -> Self {
        SummaryReportContext {
            fail_count: maybe_color(fail_count > 0, &fail, fail_count),
            not_run_count: maybe_color(not_yet_run_count > 0, &warn, not_yet_run_count),
            pass_count: maybe_color(pass_count > 0, &pass, pass_count),
            report_date: time::now(),
            test_count: maybe_color(test_count == 0, &warn, test_count),
            test_pattern: pattern.to_string(),
        }
    }
}

/// conditionally color output
fn maybe_color(condition: bool, cb: &dyn Fn(&str) -> String, count: u32) -> String {
    let s = format!(" {:05}", count);
    if condition { cb(&s) } else { s }
}

/// Render summary report to a string
pub fn render(summary_report_context: &SummaryReportContext) -> crate::error::Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template("summary_report_template", reports::SUMMARY_REPORT_TEMPLATE)?;
    let rendered = tt.render("summary_report_template", summary_report_context)?;
    Ok(rendered)
}

/// show summary template rendered output
pub fn show_summary(summary_report_context: &SummaryReportContext) -> crate::error::Result<()> {
    log::info!("summary/show_summary");
    let rendered = render(summary_report_context)?;
    println!("{}", rendered);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_render() {
        let ctx = SummaryReportContext::new(1, 2, 3, "my_test", 6);
        let output = render(&ctx).unwrap();
        assert!(output.contains("reg-rs Summary Report"));
        assert!(output.contains("failed"));
        assert!(output.contains("not yet run"));
        assert!(output.contains("passed"));
        assert!(output.contains("my_test"));
    }

    #[test]
    fn test_summary_render_zero_counts() {
        let ctx = SummaryReportContext::new(0, 0, 0, "empty", 0);
        let output = render(&ctx).unwrap();
        assert!(output.contains("empty"));
    }
}

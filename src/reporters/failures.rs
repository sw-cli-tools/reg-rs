use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::templates::reports;

/// Data for test failures report template
#[derive(Serialize)]
pub struct FailuresReportContext {
    difference_types: Vec<String>,
    differences_count: u32,
    fail_symbol: String,
    failed_test_name: String,
    required_blank: String,
    time_created: String,
    time_last_ran: String,
}

impl FailuresReportContext {
    /// build data for test failures report template
    pub fn new(
        difference_types: Vec<String>,
        differences_count: u32,
        fail_symbol: String,
        failed_test_name: String,
        time_created: String,
        time_last_ran: String,
    ) -> Self {
        FailuresReportContext {
            difference_types,
            differences_count,
            fail_symbol,
            failed_test_name,
            required_blank: crate::REQUIRED_BLANK.to_string(),
            time_created,
            time_last_ran,
        }
    }
}

/// Render test failures report to a string
pub fn render(failures_report_context: &FailuresReportContext) -> crate::error::Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template(
        "failures_report_template",
        reports::FAILURES_REPORT_TEMPLATE,
    )?;
    let rendered = tt.render("failures_report_template", failures_report_context)?;
    Ok(rendered)
}

/// show test failures template output
pub fn show_failure(failures_report_context: &FailuresReportContext) -> crate::error::Result<()> {
    log::info!("failures/show_failure");
    log::debug!("show_failure");
    let rendered = render(failures_report_context)?;
    println!("{}", rendered);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failures_render() {
        let ctx = FailuresReportContext::new(
            vec!["exit_code".to_string(), "stderr".to_string()],
            3,
            "-".to_string(),
            "broken_test".to_string(),
            "2024-01-01".to_string(),
            "2024-01-02".to_string(),
        );
        let output = render(&ctx).unwrap();
        assert!(output.contains("broken_test"));
        assert!(output.contains("2024-01-01"));
        assert!(output.contains("3"));
    }

    #[test]
    fn test_failures_render_no_difference_types() {
        let ctx = FailuresReportContext::new(
            vec![],
            1,
            "-".to_string(),
            "test1".to_string(),
            "2024-01-01".to_string(),
            "2024-01-02".to_string(),
        );
        let output = render(&ctx).unwrap();
        assert!(output.contains("test1"));
    }
}

use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::templates::reports;

/// data for test passes report template
#[derive(Serialize)]
pub struct PassesReportContext {
    pass_symbol: String,
    passed_test_name: String,
    required_blank: String,
    time_created: String,
    time_last_ran: String,
}

impl PassesReportContext {
    /// build data for test passes report template
    pub fn new(
        pass_symbol: String,
        passed_test_name: String,
        time_created: String,
        time_last_ran: String,
    ) -> Self {
        PassesReportContext {
            pass_symbol,
            passed_test_name,
            required_blank: crate::REQUIRED_BLANK.to_string(),
            time_created,
            time_last_ran,
        }
    }
}

/// Render test passes report to a string
pub fn render(passes_report_context: &PassesReportContext) -> crate::error::Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template("passes_report_template", reports::PASSES_REPORT_TEMPLATE)?;
    let rendered = tt.render("passes_report_template", passes_report_context)?;
    Ok(rendered)
}

/// show test passes rendered template output
pub fn show_passes(passes_report_context: &PassesReportContext) -> crate::error::Result<()> {
    log::info!("passes/show_passes");
    log::debug!("show_passes");
    let rendered = render(passes_report_context)?;
    println!("{}", rendered);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passes_render() {
        let ctx = PassesReportContext::new(
            "+".to_string(),
            "my_test".to_string(),
            "2024-01-01".to_string(),
            "2024-01-02".to_string(),
        );
        let output = render(&ctx).unwrap();
        assert!(output.contains("my_test"));
        assert!(output.contains("2024-01-01"));
        assert!(output.contains("2024-01-02"));
    }
}

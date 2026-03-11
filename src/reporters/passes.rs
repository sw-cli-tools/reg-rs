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
            required_blank: " ".to_string(),
            time_created,
            time_last_ran,
        }
    }
}

/// show test passes rendered template output
pub fn show_passes(
    passes_report_context: &PassesReportContext,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("passes/show_passes");
    log::debug!("show_passes");
    let mut tt = TinyTemplate::new();
    tt.add_template("passes_report_template", reports::PASSES_REPORT_TEMPLATE)?;
    let rendered = tt.render("passes_report_template", &passes_report_context)?;
    println!("{}", rendered);
    Ok(())
}

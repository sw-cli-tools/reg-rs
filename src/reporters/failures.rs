use tinytemplate::TinyTemplate;

use crate::templates::reports;

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
            required_blank: " ".to_string(),
            time_created,
            time_last_ran,
        }
    }
}

pub fn show_failure(
    failures_report_context: &FailuresReportContext,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("failures/show_failure");
    md!("testing");
    let mut tt = TinyTemplate::new();
    tt.add_template(
        "failures_report_template",
        reports::FAILURES_REPORT_TEMPLATE,
    )?;
    let rendered = tt.render("failures_report_template", &failures_report_context)?;
    println!("{}", rendered);
    Ok(())
}

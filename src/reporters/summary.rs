use tinytemplate::TinyTemplate;

use crate::templates::reports;
use crate::time;
use crate::util::{fail, pass, warn};

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
    pub fn new(
        fail_count: u32,
        not_yet_run_count: u32,
        pass_count: u32,
        pattern: &str,
        test_count: u32,
    ) -> Self {
        SummaryReportContext {
            fail_count: fail(&format!(" {:05}", fail_count)),
            not_run_count: warn(&format!(" {:05}", not_yet_run_count)),
            pass_count: pass(&format!(" {:05}", pass_count)),
            report_date: time::now(),
            test_count: format!(" {:05}", test_count),
            test_pattern: pattern.to_string(),
        }
    }
}

pub fn show_summary(
    summary_report_context: &SummaryReportContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tt = TinyTemplate::new();
    tt.add_template("summary_report_template", reports::SUMMARY_REPORT_TEMPLATE)?;
    let rendered = tt.render("summary_report_template", &summary_report_context)?;
    println!("{}", rendered);
    Ok(())
}

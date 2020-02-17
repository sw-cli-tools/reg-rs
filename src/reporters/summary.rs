use tinytemplate::TinyTemplate;

use crate::config;
use crate::templates::reports;
use crate::time;

#[derive(Serialize)]
struct SummaryReportContext {
    fail_count: String,
    not_run_count: String,
    pass_count: String,
    report_date: String,
    test_count: String,
    test_pattern: String,
}

pub fn show_summary(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    md!("incomplete");
    let mut tt = TinyTemplate::new();
    tt.add_template("summary_report_template", reports::SUMMARY_REPORT_TEMPLATE)?;
    let summary_report_context = SummaryReportContext {
        fail_count: format!(" {:05}", 0),
        not_run_count: format!(" {:05}", 0),
        pass_count: format!(" {:05}", 0),
        report_date: time::now(),
        test_count: format!(" {:05}", 0),
        test_pattern: config.extract_pattern().to_string(),
    };
    let rendered = tt.render("summary_report_template", &summary_report_context)?;
    println!("{}", rendered);
    Ok(())

}

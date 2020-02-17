use tinytemplate::TinyTemplate;

use crate::templates::reports;

#[derive(Serialize)]
pub struct DetailsReportContext {
    failed_test_names: Vec<String>, 
    not_yet_run_test_names: Vec<String>,
    passed_test_names: Vec<String>,
}

impl DetailsReportContext {
    pub fn new(
        failed_test_names: Vec<String>,
        not_yet_run_test_names: Vec<String>,
        passed_test_names: Vec<String>,
    ) -> Self {
        DetailsReportContext {
            failed_test_names,
            not_yet_run_test_names,
            passed_test_names,
        }
    }
}

pub fn show_details(details_report_context: &DetailsReportContext
) -> Result<(), Box<dyn std::error::Error>> {
    md!("testing");
    let mut tt = TinyTemplate::new();
    tt.add_template("details_report_template", reports::DETAILS_REPORT_TEMPLATE)?;
    let rendered = tt.render("details_report_template", &details_report_context)?;
    println!("{}", rendered);
    Ok(())
}

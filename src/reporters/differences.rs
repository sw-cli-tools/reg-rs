use log;
use tinytemplate::TinyTemplate;

use crate::templates::reports;

#[derive(Debug, Serialize)]
pub struct DisplayDifference {
    pub type_name: String,
    pub chunk: String,
}

#[derive(Debug, Serialize)]
pub struct DifferencesReportContext {
    differences: Vec<DisplayDifference>,
    failed_test_name: String,
}

impl DifferencesReportContext {
    pub fn new(
        differences: Vec<DisplayDifference>,
        failed_test_name: String,
    ) -> Self {
        DifferencesReportContext {
            differences,
            failed_test_name,

        }
    }
}

pub fn show_differences(
    differences_report_context: &DifferencesReportContext,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("differences/show_differences");
    md!("testing");
    let mut tt = TinyTemplate::new();
    tt.add_template(
        "differences_report_template",
        reports::DIFFERENCES_REPORT_TEMPLATE,
    )?;
    let rendered = tt.render("differences_report_template", &differences_report_context)?;
    println!("{}", rendered);
    Ok(())
}



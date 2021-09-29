use tinytemplate::TinyTemplate;

use crate::templates::reports;

/// Describe a difference
#[derive(Debug, Serialize)]
pub struct DisplayDifference {
    /// difference type
    pub type_name: String,
    /// difference data
    pub chunk: String,
}

/// data for a differences report template
#[derive(Debug, Serialize)]
pub struct DifferencesReportContext {
    /// list of differences
    differences: Vec<DisplayDifference>,
    /// failed test name
    failed_test_name: String,
}

impl DifferencesReportContext {
    /// build new difference report template data
    pub fn new(differences: Vec<DisplayDifference>, failed_test_name: String) -> Self {
        DifferencesReportContext {
            differences,
            failed_test_name,
        }
    }
}

/// show test result differences
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

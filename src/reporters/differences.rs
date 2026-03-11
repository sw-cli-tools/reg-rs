use serde::Serialize;
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

/// Render test result differences to a string
pub fn render(
    differences_report_context: &DifferencesReportContext,
) -> crate::error::Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template(
        "differences_report_template",
        reports::DIFFERENCES_REPORT_TEMPLATE,
    )?;
    let rendered = tt.render("differences_report_template", differences_report_context)?;
    Ok(rendered)
}

/// show test result differences
pub fn show_differences(
    differences_report_context: &DifferencesReportContext,
) -> crate::error::Result<()> {
    log::info!("differences/show_differences");
    log::debug!("show_differences");
    let rendered = render(differences_report_context)?;
    println!("{}", rendered);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_differences_render() {
        let ctx = DifferencesReportContext::new(
            vec![
                DisplayDifference {
                    type_name: "stderr add".to_string(),
                    chunk: "error output".to_string(),
                },
                DisplayDifference {
                    type_name: "stdout add".to_string(),
                    chunk: "new output".to_string(),
                },
            ],
            "failing_test".to_string(),
        );
        let output = render(&ctx).unwrap();
        assert!(output.contains("Differences"));
        assert!(output.contains("stderr add"));
        assert!(output.contains("error output"));
    }

    #[test]
    fn test_differences_render_empty() {
        let ctx = DifferencesReportContext::new(vec![], "test1".to_string());
        let output = render(&ctx).unwrap();
        assert!(output.contains("Differences"));
    }
}

use tinytemplate::TinyTemplate;

use crate::db;
use crate::diff;
use crate::reporters::failures;
use crate::templates::reports;

#[derive(Serialize)]
pub struct DetailsReportContext {
    failed_test_names: Vec<String>,
    no_failed_tests: bool,
    no_not_yet_run_tests: bool,
    no_passed_tests: bool,
    not_yet_run_test_names: Vec<String>,
    passed_test_names: Vec<String>,
}

impl DetailsReportContext {
    pub fn new(
        failed_test_names: Vec<String>,
        no_failed_tests: bool,
        no_not_yet_run_tests: bool,
        no_passed_tests: bool,
        not_yet_run_test_names: Vec<String>,
        passed_test_names: Vec<String>,
    ) -> Self {
        DetailsReportContext {
            failed_test_names,
            no_failed_tests,
            no_not_yet_run_tests,
            no_passed_tests,
            not_yet_run_test_names,
            passed_test_names,
        }
    }
}

pub fn show_details(
    details_report_context: &DetailsReportContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tt = TinyTemplate::new();
    tt.add_template("details_report_template", reports::DETAILS_REPORT_TEMPLATE)?;
    let rendered = tt.render("details_report_template", &details_report_context)?;
    println!("{}", rendered);
    show_failures(&details_report_context)?;
    Ok(())
}

fn show_failures(
    details_report_context: &DetailsReportContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let failed_test_names = &details_report_context.failed_test_names;
    if 0 < *&failed_test_names.len() {
        println!("Failures:");
    }
    for test in failed_test_names {
        md!(&test);
        let original_result = db::read_original_results(&test)?;
        md!(&original_result);
        let latest_result = db::read_latest_results(&test)?;
        md!(&latest_result);
        let differences = db::read_differences(&test)?;
        md!(&differences);
        let mut difference_types = vec![];
        let same_count =
            db::difference_count_by_type(&test, diff::RegressionType::StderrSame as u8)?
                + db::difference_count_by_type(&test, diff::RegressionType::StdoutSame as u8)?;
        let differences_count = *&differences.len() as u32 - same_count;
        if 0 < db::difference_count_by_type(&test, diff::RegressionType::ActualCode as u8)? {
            difference_types.push("exit_code".to_string());
        }
        if 0 < db::difference_count_by_type(&test, diff::RegressionType::StderrAdd as u8)?
            || 0 < db::difference_count_by_type(&test, diff::RegressionType::StderrRemove as u8)?
        {
            difference_types.push("stderr".to_string());
        }
        if 0 < db::difference_count_by_type(&test, diff::RegressionType::StdoutAdd as u8)?
            || 0 < db::difference_count_by_type(&test, diff::RegressionType::StdoutRemove as u8)?
        {
            difference_types.push("stdout".to_string());
        }
        failures::show_failure(&failures::FailuresReportContext::new(
            difference_types,
            differences_count,
            test.to_string(),
            original_result.time_created.to_string(),
            latest_result.time_created.to_string(),
        ))?;
    }
    Ok(())
}

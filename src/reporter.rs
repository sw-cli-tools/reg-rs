use tinytemplate::TinyTemplate;

use crate::config;
use crate::db;
use std::collections::HashMap;
use crate::finder;
use crate::templates::reports;

#[derive(Serialize)]
struct DetailsContext {
    heading: String,
    values: Vec<String>,
}

#[derive(Serialize)]
struct DifferencesContext {
    heading: String,
    values: Vec<HashMap<String, String>>,
}

#[derive(Serialize)]
struct ReportContext {
    heading: String,
    tests: Vec<String>,
    details: DetailsContext,
    differences: DifferencesContext,
}

pub fn generate(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let test_names = finder::discover(&config)?;
    if config.debug {
        md!(&config);
    }
    let details_context = DetailsContext {
        heading: "Details".to_string(),
        values: vec![],
    };
    let differences_context = DifferencesContext {
        heading: "Differences".to_string(),
        values: vec![],
    };
    let mut report_context = ReportContext {
        heading: "Regression Test Tool - test results report".to_string(),
        tests: vec![],
        details: details_context,
        differences: differences_context,
    };
    for test_name in test_names.found {
        md!(("found", &test_name));
        let original_result = db::read_original_results(&test_name)?;
        md!(&original_result);
        let latest_result = db::read_latest_results(&test_name)?;
        md!(&latest_result);
        let differences = db::read_differences(&test_name)?;
        md!(&differences);
        report_context.tests.push(original_result.name);
        // todo push differences?  determine pass/fail?
        report_context.details.values.push("blah".to_string());
        for difference in differences {
            let mut map = std::collections::HashMap::new();
            map.insert("type".to_string(), difference.0);
            map.insert("chunk".to_string(), difference.1);
            report_context.differences.values.push(map);
        }
    }
    let mut tt = TinyTemplate::new();
    tt.add_template("report_template", reports::REPORT_TEMPLATE)?;
    tt.add_template("differences_template", reports::DIFFERENCES_TEMPLATE)?;
    tt.add_template("details_template", reports::DETAILS_TEMPLATE)?;
    let rendered = tt.render("report_template", &report_context)?;
    println!("{}", rendered);
    Ok(())
}

use tinytemplate::TinyTemplate;

use crate::config;
use crate::db;
use crate::finder;
use crate::templates::reports;

#[derive(Serialize)]
struct DetailsContext {
    heading: String,
    values: Vec<String>,
}

#[derive(Serialize)]
struct ReportContext {
    heading: String,
    tests: Vec<String>,
    details: DetailsContext,
}

pub fn generate(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let tests = finder::discover(&config)?;
    if config.debug {
        md!(&config);
    }
    let details_context = DetailsContext {
        heading: "Details".to_string(),
        values: vec![],
    };
    let mut report_context = ReportContext {
        heading: "Regression Test Tool - test results report".to_string(),
        tests: vec![],
        details: details_context,
    };
    for test in tests.found {
        md!(("found", &test));
        let result = db::read_original_results(&test)?;
        md!(&result);
        report_context.tests.push(result.name);
        report_context.details.values.push("blah".to_string());
    }
    let mut tt = TinyTemplate::new();
    tt.add_template("report_template", reports::REPORT_TEMPLATE)?;
    tt.add_template("details_template", reports::DETAILS_TEMPLATE)?;
    let rendered = tt.render("report_template", &report_context)?;
    println!("{}", rendered);
    Ok(())
}

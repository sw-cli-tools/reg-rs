extern crate tinytemplate;

use tinytemplate::TinyTemplate;

use crate::config;
use crate::db;
use crate::runner;

#[derive(Serialize)]
struct Context {
    heading: String,
    tests: Vec<String>,
}

static TEMPLATE: &str = "
{heading}
===

Test {{ for name in tests }} {name} {{ endfor }}
";

pub fn generate(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let tests = runner::discover(&config)?;
    if config.debug {
        md!(&config);
    }
    let mut context = Context {
        heading: "Regression Test Tool - retst results report".to_string(),
        tests: vec![],
    };
    for test in tests.found {
        md!(("found", &test));
        let result = db::open_read(&test)?;
        md!(&result);
        context.tests.push(result.name);
    }
    let mut tt = TinyTemplate::new();
    tt.add_template("report", TEMPLATE)?;
    let rendered = tt.render("report", &context)?;
    println!("{}", rendered);
    Ok(())
}

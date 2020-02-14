use string_template::Template;
use std::collections::HashMap;
use crate::config;
use crate::db;
use crate::runner;

pub fn generate(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let tests = runner::discover(&config)?;
    if config.debug {
        md!(&config);
    }
    for test in tests.found {
        md!(("found", &test));
        let result = db::open_read(&test)?;
        md!(&result);
        let template = Template::new("
Regression Test Tool report
===

Test name {{test_name}} created at {{created_at}}
");
        let mut template_args = HashMap::new();
        template_args.insert("test_name", &*result.name);
        template_args.insert("created_at", &*result.time_created);
        let s = template.render(&template_args);
        println!("{}", s); // TODO generate header and then row per test result
    }
    Ok(())
}

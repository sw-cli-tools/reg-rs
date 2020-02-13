use walkdir::{DirEntry, Error, WalkDir};

use crate::config;

#[derive(Debug)]
pub struct Tests {
    found: Vec<String>,
}

fn execute_closure(
    closure_argument: &mut dyn FnMut(&mut Vec<String>, String),
    acc: &mut Vec<String>,
    value: String,
) {
    closure_argument(acc, value);
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn subject() -> Result<Tests, Error> {
    let mut tests = Tests { found: vec![] };
    let mut closure_variable = |acc: &mut Vec<String>, val: String| {
        if val.contains(".tdb") {
            acc.push(val);
        }
    };
    let walker = WalkDir::new("data").into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        execute_closure(
            &mut closure_variable,
            &mut tests.found,
            format!("{:?}", entry?.path().display()),
        );
    }
    Ok(tests)
}

pub fn discover(config: &config::Config) -> Result<Tests, Box<dyn std::error::Error>> {
    let tests = subject()?;
    if config.debug {
        dbg!(&tests);
    }
    Ok(tests)
}
pub fn run(config: &config::Config, tests: &Tests) -> Result<(), Box<dyn std::error::Error>> {
    if config.debug {
        dbg!(&config);
        dbg!(&tests);
    }
    Ok(())
}

use walkdir::{DirEntry, Error, WalkDir};

use crate::config;

#[derive(Debug)]
pub struct TestNames {
    pub found: Vec<String>,
}

#[derive(Debug)]
pub struct TestResults {
    pub id: i32,
    pub name: String,
    pub command: String,
    pub time_created: String,
    pub exit_code: i32,
    pub stderr: String,
    pub stdout: String,
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
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn subject(pattern: String) -> Result<TestNames, Error> {
    let mut tests = TestNames { found: vec![] };
    let mut closure_variable = |acc: &mut Vec<String>, val: String| {
        if val.contains(&pattern) {
            acc.push(val);
        }
    };
    let walker = WalkDir::new("data").into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        execute_closure(
            &mut closure_variable,
            &mut tests.found,
            format!("{}", entry?.path().display()),
        );
    }
    Ok(tests)
}

pub fn discover(config: &config::Config) -> Result<TestNames, Box<dyn std::error::Error>> {
    let tests = subject(config.extract_pattern().to_string())?;
    if config.debug {
        md!(&tests);
    }
    Ok(tests)
}

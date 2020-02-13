use chrono::Local;
use walkdir::{DirEntry, Error, WalkDir};

use crate::config;
use crate::process;
use crate::runner;

#[derive(Debug)]
pub struct TestNames { 
    found: Vec<String>,
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
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn subject() -> Result<TestNames, Error> {
    let mut tests = TestNames { found: vec![] };
    let mut closure_variable = |acc: &mut Vec<String>, val: String| {
        if val.contains(".tdb") {  // TODO match pattern
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

pub fn discover(config: &config::Config) -> Result<TestNames, Box<dyn std::error::Error>> {
    let tests = subject()?; // TODO pass pattern
    if config.debug {
        md!(&tests);
    }
    Ok(tests)
}

pub fn run_many(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let tests = runner::discover(&config)?; 
    if config.debug {
        md!(&config);
        md!(&tests);
    }
    // if dry-run, print summary
    // else for each test, run it
    Ok(())
}

pub fn run_one(test_name: &String, command: &String) -> Result<TestResults, Box<dyn std::error::Error>> {
    let (exit_code, stderr, stdout) = process::exec(command.to_string())?;
    let test = TestResults {
        id: 0,
        name: (&test_name).to_string(),
        command: (&command).to_string(),
        time_created: now(),
        exit_code,
        stderr,
        stdout,
    };
    Ok(test)
}

fn now() -> String {
    let date = Local::now();
    format!("{}", date.format("%Y-%m-%dT%H:%M:%S"))
}

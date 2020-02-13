use walkdir::{DirEntry, Error, WalkDir};

use crate::config;
use crate::process;

#[derive(Debug)]
pub struct Tests { // TODO rename TestNames
    found: Vec<String>,
}

#[derive(Debug)]
pub struct Test { // TODO rename TestResults
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
pub fn run_many(config: &config::Config, tests: &Tests) -> Result<(), Box<dyn std::error::Error>> {
    if config.debug {
        dbg!(&config);
        dbg!(&tests);
    }
    Ok(())
}
pub fn run_one(test_name: &String, command: &String) -> Result<Test, Box<dyn std::error::Error>> {
    let (exit_code, stderr, stdout) = process::exec(command.to_string())?;
    let test = Test {
        id: 0,
        name: (&test_name).to_string(),
        command: (&command).to_string(),
        time_created: "now".to_string(), // TODO timestamp
        exit_code,
        stderr,
        stdout,
    };
    Ok(test)
}

use walkdir::{DirEntry, Error, WalkDir};

use crate::args;
use crate::config;
use crate::db;
use crate::diff;
use crate::process;
use crate::runner;
use crate::time;

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

pub fn run_many(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let tests = runner::discover(&config)?;
    if config.debug {
        md!(&config);
        md!(&tests);
    }
    let dry_run = match &config.mode {
        args::Subcommands::Run { dry_run, .. } => dry_run,
        _ => &false,
    };
    for test in tests.found {
        let prior_test_results = db::open_read(&test)?;
        let maybe_regression = run_one(&test, &prior_test_results.command, *dry_run)?;
        if let Some(latest_test_results) = maybe_regression {
            // compare exit_code
            if prior_test_results.exit_code != latest_test_results.exit_code {
                md!((prior_test_results.exit_code,
                     latest_test_results.exit_code));
            }
            if let Some(stderr_diff) =
                diff::compare(&prior_test_results.stderr, &latest_test_results.stderr)
            {
                md!(stderr_diff);
            }
            if let Some(stdout_diff) =
                diff::compare(&prior_test_results.stdout, &latest_test_results.stdout)
            {
                md!(stdout_diff);
            }
        }
    }
    Ok(())
}

pub fn run_one(
    test_name: &str,
    command: &str,
    dry_run: bool,
) -> Result<Option<TestResults>, Box<dyn std::error::Error>> {
    if dry_run {
        println!("dry-run: test name: {}, command: {}", test_name, command);
        Ok(None)
    } else {
        let (exit_code, stderr, stdout) = process::exec(command.to_string())?;
        let test = TestResults {
            id: 0,
            name: test_name.to_string(),
            command: command.to_string(),
            time_created: time::now(),
            exit_code,
            stderr,
            stdout,
        };
        Ok(Some(test))
    }
}

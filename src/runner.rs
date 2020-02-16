use crate::args;
use crate::config;
use crate::db;
use crate::diff;
use crate::finder;
use crate::process;
use crate::time;

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

pub fn run_many(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let tests = finder::discover(&config)?;
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

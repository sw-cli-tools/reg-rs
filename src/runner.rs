use crate::config;
use crate::db;
use crate::diff;
use crate::finder;
use crate::process;
use crate::queries;
use crate::time;

#[derive(Debug)]
pub struct TestResults {
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
    for test in tests.found {
        let prior_test_result = db::read_original_results(&test)?;
        let maybe_regression = run_one(&test, &prior_test_result.command, config.is_dry_run())?;
        if let Some(latest_test_result) = maybe_regression {
            let db_name = &test;
            diff::process_differences(&db_name, &prior_test_result, &latest_test_result)?;
            db::drop_latest_results(&db_name)?;
            db::store_results(
                &db_name,
                &latest_test_result,
                queries::StatementContext::latest(),
            )?;
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
            name: test_name.to_string(),
            command: command.to_string(),
            time_created: time::now(),
            exit_code,
            stderr,
            stdout,
        };
        // db write regression results (maybe_create, update)
        Ok(Some(test))
    }
}

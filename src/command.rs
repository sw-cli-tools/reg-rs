use crate::config;
use crate::db;
use crate::finder;
use crate::queries;
use crate::reporter;
use crate::runner;

pub fn create_original(
    config: &config::Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    md!(&config);
    let (test, command) = config.extract_test_and_command().unwrap();
    if let Some(test_result) = runner::run_one(&test, &command, false)? {
        let db_name = test;
        db::reset_differences(&db_name)?;
        db::store_results(
            &db_name,
            &test_result,
            queries::StatementContext::original(),
        )?;
    }
    Ok(())
}

pub fn update_latest(
    config: &config::Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    runner::run_many(&config)?;
    Ok(())
}

pub fn remove_all(config: &config::Config) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let tests = finder::discover(&config)?;
    for test in tests.found {
        db::drop_all_results(&test)?;
    }
    Ok(())
}

pub fn report_latest(
    config: &config::Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    reporter::generate(&config)?;
    Ok(())
}

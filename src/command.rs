use log;

use crate::config;
use crate::db;
use crate::finder;
use crate::queries;
use crate::reporters::generate_reports;
use crate::runner;
use crate::status;

pub fn create_original(
    config: &config::Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log::info!("command/create_original");
    md!(&config);
    let (test, command) = config.extract_test_and_command().unwrap();
    if let Some(test_result) = runner::run_one(&test, &command, false)? {
        let db_name = test;
        db::reset_differences(&db_name)?;
        db::reset_latest_results(&db_name)?;
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
    log::info!("command/update_latest");
    runner::run_many(&config)?;
    Ok(())
}

pub fn remove_all(config: &config::Config) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log::info!("command/remove_all");
    let tests = finder::discover(config.extract_pattern().to_string())?;
    md!(&tests);
    for test in tests.found {
        db::drop_all_results(&test)?;
    }
    log::info!("command/remove_all done");
    Ok(())
}

pub fn report_latest(
    config: &config::Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log::info!("command/report_latest");
    generate_reports(&config)?;
    log::info!("command/report_latest done");
    Ok(())
}
pub fn status_server(
    config: &config::Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log::info!("command/status_server");
    status::start_client(&config)?; 
    status::start_server(&config)?; // loops
    Ok(())
}    

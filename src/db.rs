use rusqlite::{Result};
use crate::config;
use crate::runner;
use crate::sqlite;

pub fn create(config: &config::Config)
              -> std::result::Result<(), Box<dyn std::error::Error>> {
    if super::DEBUG {
        dbg!(&config);
    }
    let (test, command) = config::extract_test_and_command(config).unwrap();
    let test_result = runner::run_one(&test, &command)?;
    let db_name = test;
    open_maybe_create_write(&db_name, test_result);
    Ok(())
}

fn open_maybe_create_write(db_name: &str, test: runner::Test) -> Result<()> {
    sqlite::maybe_create_table(db_name)?;
    sqlite::write(db_name, test)?;
    Ok(())
}

fn open_read(db_name: &str) -> Result<runner::Test> {
    Ok(sqlite::open_query(&db_name, &db_name)?) // TODO separate db and test names
}

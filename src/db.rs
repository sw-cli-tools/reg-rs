use file_lock::FileLock;
use rusqlite::Result;

use crate::diff;
use crate::queries;
use crate::runner;
use crate::sqlite;
use crate::templates::statements;

const BLOCKING: bool = true;
const WRITING: bool = true;

pub(crate) fn store_results(
    db_name: &str,
    test_results: &runner::TestResults,
    statement_context: queries::StatementContext,
) -> Result<()> {
    log::info!("db/store_results {}", &db_name);
    let filelock = match FileLock::lock(db_name, BLOCKING, WRITING) {
        Ok(lock) => lock,
        Err(e) => panic!("unable to get lock, e={}", e),
    };
    sqlite::create_table(
        db_name,
        &queries::get_statement(
            &statement_context,
            statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
        ),
    )?;
    sqlite::write_results(
        db_name,
        test_results,
        &queries::get_statement(
            &statement_context,
            statements::INSERT_TEST_RESULTS_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(())
}

pub fn read_original_results(db_name: &str) -> Result<runner::TestResults> {
    log::info!("db/read_original_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    md!(&db_name);
    md!(&queries::StatementContext::original());
    md!(&statements::SELECT_TEST_RESULTS_TEMPLATE);
    let results = sqlite::read_results(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::SELECT_TEST_RESULTS_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(results)
}

pub fn reset_latest_results(db_name: &str) -> Result<()> {
    log::info!("db/reset_latest_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    sqlite::drop_table(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            statements::DROP_TABLE_TEMPLATE,
        ),
    )?;
    sqlite::create_table(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(())
}

pub fn drop_all_results(db_name: &str) -> Result<()> {
    log::info!("db/drop_all_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    sqlite::drop_table(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::DROP_TABLE_TEMPLATE,
        ),
    )?;
    reset_latest_results(db_name)?;
    filelock.unlock().unwrap();
    Ok(())
}

pub fn reset_differences(db_name: &str) -> Result<()> {
    log::info!("db/reset_differences {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    sqlite::drop_table(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::DROP_TABLE_TEMPLATE,
        ),
    )?;
    sqlite::create_table(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::CREATE_DIFFERENCES_TABLE_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(())
}

pub fn store_difference(
    db_name: &str,
    difference_type: diff::RegressionType,
    difference_chunk: &str,
) -> Result<()> {
    log::info!("db/store_difference {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    let difference_type = (difference_type as usize).to_string();
    sqlite::write_difference(db_name, &difference_type, difference_chunk)?;
    filelock.unlock().unwrap();
    Ok(())
}

pub fn read_latest_results(db_name: &str) -> Result<runner::TestResults> {
    log::info!("db/read_latest_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    let results = sqlite::read_results(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            statements::SELECT_TEST_RESULTS_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(results)
}

pub fn read_differences(db_name: &str) -> Result<Vec<(String, String)>> {
    log::info!("db/read_differences {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    let result = sqlite::read_differences(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::SELECT_DIFFERENCES_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(result)
}

pub fn count_differences(db_name: &str) -> Result<u32> {
    log::info!("db/count_differences {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    let result = sqlite::count_rows(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(result)
}

pub fn count_latest_results(db_name: &str) -> Result<u32> {
    log::info!("db/count_latest_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    let result = sqlite::count_rows(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(result)
}

pub fn difference_count_by_type(db_name: &str, difference_type: u8) -> Result<u32> {
    log::info!("db/difference_count_by_type {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    let results = sqlite::count_differences_by_type(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::difference_count_by_type(difference_type),
            statements::COUNT_DIFF_TYPE_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(results)
}

pub fn clear_differences(db_name: &str) -> Result<()> {
    log::info!("db/clear_differences {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    sqlite::delete_all_rows(
        db_name, 
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::DELETE_ALL_ROWS_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(())
}

pub fn clear_latest_results(db_name: &str) -> Result<()> {
    log::info!("db/clear_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING).unwrap();
    sqlite::delete_all_rows(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            statements::DELETE_ALL_ROWS_TEMPLATE,
        ),
    )?;
    filelock.unlock().unwrap();
    Ok(())
}

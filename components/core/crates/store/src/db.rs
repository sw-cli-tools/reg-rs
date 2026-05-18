use file_lock::FileLock;

use crate::queries;
use crate::queries::statements;
use crate::sqlite;
use reg_rs_types::error::{RegError, Result};
use reg_rs_types::types::TestResults;

const BLOCKING: bool = true;
const WRITING: bool = true;

/// Get the path to the lock file for a given database file.
pub(crate) fn lock_file_path(db_name: &str) -> String {
    format!("{}.{}", db_name, reg_rs_types::constants::LOCK_EXTENSION)
}

/// Execute a closure while holding a file lock on the database.
pub(crate) fn with_lock<T, F>(db_name: &str, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let filelock = FileLock::lock(&lock_file_path(db_name), BLOCKING, WRITING)
        .map_err(|e| RegError::FileLock(format!("unable to get lock for {db_name}: {e}")))?;
    let result = f();
    filelock
        .unlock()
        .map_err(|e| RegError::FileLock(format!("unable to unlock {db_name}: {e}")))?;
    result
}

/// Store test results in a table under file lock
pub fn store_results(
    db_name: &str,
    test_results: &TestResults,
    statement_context: queries::StatementContext,
) -> Result<()> {
    log::info!("db/store_results {}", &db_name);
    with_lock(db_name, || {
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
            &queries::get_statement(&statement_context, statements::INSERT_TEST_RESULTS_TEMPLATE),
        )?;
        Ok(())
    })
}

/// Read first time test results
pub fn read_original_results(db_name: &str) -> Result<TestResults> {
    log::info!("db/read_original_results {}", &db_name);
    with_lock(db_name, || {
        log::debug!("read_original_results db: {}", &db_name);
        sqlite::read_results(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::SELECT_TEST_RESULTS_TEMPLATE,
            ),
        )
    })
}

/// Read latest test results
pub fn read_latest_results(db_name: &str) -> Result<TestResults> {
    log::info!("db/read_latest_results {}", &db_name);
    with_lock(db_name, || {
        sqlite::read_results(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::SELECT_TEST_RESULTS_TEMPLATE,
            ),
        )
    })
}

/// Atomically clear and replace latest test results under a single file lock.
pub fn replace_latest_results(db_name: &str, test_results: &TestResults) -> Result<()> {
    log::info!("db/replace_latest_results {}", &db_name);
    with_lock(db_name, || {
        sqlite::create_table(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
            ),
        )?;
        sqlite::delete_all_rows(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::DELETE_ALL_ROWS_TEMPLATE,
            ),
        )?;
        sqlite::write_results(
            db_name,
            test_results,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::INSERT_TEST_RESULTS_TEMPLATE,
            ),
        )?;
        Ok(())
    })
}

/// Reset latest test results
pub fn reset_latest_results(db_name: &str) -> Result<()> {
    log::info!("db/reset_latest_results {}", &db_name);
    with_lock(db_name, || drop_and_recreate_latest(db_name))
}

/// Drop all test results
pub fn drop_all_results(db_name: &str) -> Result<()> {
    log::info!("db/drop_all_results {}", &db_name);
    with_lock(db_name, || {
        sqlite::drop_table(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::DROP_TABLE_TEMPLATE,
            ),
        )?;
        drop_and_recreate_latest(db_name)
    })
}

/// Drop and recreate the latest results table (no lock).
pub(crate) fn drop_and_recreate_latest(db_name: &str) -> Result<()> {
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
    )
}

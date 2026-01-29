use file_lock::FileLock;

use crate::diff;
use crate::error::{Result, RttError};
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
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
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
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(())
}

/// read first time test results
pub fn read_original_results(db_name: &str) -> Result<runner::TestResults> {
    log::info!("db/read_original_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
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
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(results)
}

/// Internal helper to reset latest test results without acquiring lock
/// Caller must hold the lock
fn reset_latest_results_internal(db_name: &str) -> Result<()> {
    log::info!("db/reset_latest_results_internal {}", &db_name);
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
    Ok(())
}

/// reset latest test results
pub fn reset_latest_results(db_name: &str) -> Result<()> {
    log::info!("db/reset_latest_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
    reset_latest_results_internal(db_name)?;
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(())
}

/// drop all test results
pub fn drop_all_results(db_name: &str) -> Result<()> {
    log::info!("db/drop_all_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
    sqlite::drop_table(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::DROP_TABLE_TEMPLATE,
        ),
    )?;
    // Use internal function to avoid deadlock - we already hold the lock
    reset_latest_results_internal(db_name)?;
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(())
}

/// reset test result differences
pub fn reset_differences(db_name: &str) -> Result<()> {
    log::info!("db/reset_differences {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
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
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(())
}

/// store test result differences
pub fn store_difference(
    db_name: &str,
    difference_type: diff::RegressionType,
    difference_chunk: &str,
) -> Result<()> {
    log::info!("db/store_difference {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
    let difference_type = (difference_type as usize).to_string();
    sqlite::write_difference(db_name, &difference_type, difference_chunk)?;
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(())
}

/// read latest test results
pub fn read_latest_results(db_name: &str) -> Result<runner::TestResults> {
    log::info!("db/read_latest_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
    let results = sqlite::read_results(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            statements::SELECT_TEST_RESULTS_TEMPLATE,
        ),
    )?;
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(results)
}

/// read test result differences
pub fn read_differences(db_name: &str) -> Result<Vec<(String, String)>> {
    log::info!("db/read_differences {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
    let result = sqlite::read_differences(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::SELECT_DIFFERENCES_TEMPLATE,
        ),
    )?;
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(result)
}

/// count test result differences
pub fn count_differences(db_name: &str) -> Result<u32> {
    log::info!("db/count_differences {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
    let result = sqlite::count_rows(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )?;
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(result)
}

/// count latest test results
pub fn count_latest_results(db_name: &str) -> Result<u32> {
    log::info!("db/count_latest_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
    let result = sqlite::count_rows(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )?;
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(result)
}

/// count test differences by type
pub fn difference_count_by_type(db_name: &str, difference_type: u8) -> Result<u32> {
    log::info!("db/difference_count_by_type {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
    let results = sqlite::count_differences_by_type(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::difference_count_by_type(difference_type),
            statements::COUNT_DIFF_TYPE_TEMPLATE,
        ),
    )?;
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(results)
}

/// clear test result differences
pub fn clear_differences(db_name: &str) -> Result<()> {
    log::info!("db/clear_differences {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
    sqlite::delete_all_rows(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::DELETE_ALL_ROWS_TEMPLATE,
        ),
    )?;
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(())
}

/// clear latest test results
pub fn clear_latest_results(db_name: &str) -> Result<()> {
    log::info!("db/clear_results {}", &db_name);
    let filelock = FileLock::lock(db_name, BLOCKING, WRITING)
        .map_err(|e| RttError::FileLock(format!("unable to get lock for {}: {}", db_name, e)))?;
    sqlite::delete_all_rows(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            statements::DELETE_ALL_ROWS_TEMPLATE,
        ),
    )?;
    filelock
        .unlock()
        .map_err(|e| RttError::FileLock(format!("unable to unlock {}: {}", db_name, e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_results(name: &str) -> runner::TestResults {
        runner::TestResults {
            name: name.to_string(),
            command: "echo hello".to_string(),
            time_created: "2024-01-01 12:00:00".to_string(),
            exit_code: 0,
            stderr: "".to_string(),
            stdout: "hello\n".to_string(),
        }
    }

    /// Test sqlite operations directly without file locking to verify core functionality
    #[test]
    fn test_sqlite_store_and_read_results() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test1.db");
        let db_path_str = db_path.to_str().unwrap();

        let test_results = create_test_results("test1");

        // Create table and store results directly
        sqlite::create_table(
            db_path_str,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
            ),
        )
        .unwrap();

        sqlite::write_results(
            db_path_str,
            &test_results,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::INSERT_TEST_RESULTS_TEMPLATE,
            ),
        )
        .unwrap();

        // Read them back
        let read_results = sqlite::read_results(
            db_path_str,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::SELECT_TEST_RESULTS_TEMPLATE,
            ),
        )
        .unwrap();

        assert_eq!(read_results.name, "test1");
        assert_eq!(read_results.command, "echo hello");
        assert_eq!(read_results.exit_code, 0);
        assert_eq!(read_results.stdout, "hello\n");
    }

    /// Test that reset_latest_results_internal is correctly factored out
    /// to prevent deadlock in drop_all_results
    #[test]
    fn test_reset_internal_factored_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test2.db");
        let db_path_str = db_path.to_str().unwrap();

        // Create latest table directly
        sqlite::create_table(
            db_path_str,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
            ),
        )
        .unwrap();

        // Calling internal reset should work without lock issues
        reset_latest_results_internal(db_path_str).unwrap();

        // Verify the table was recreated (should not error)
        let count = sqlite::count_rows(
            db_path_str,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::COUNT_TABLE_ROWS_TEMPLATE,
            ),
        )
        .unwrap();
        assert_eq!(count, 0);
    }

    /// Test sqlite count and clear operations
    #[test]
    fn test_sqlite_count_and_clear() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test3.db");
        let db_path_str = db_path.to_str().unwrap();

        let test_results = create_test_results("test3");

        // Create and populate latest table
        sqlite::create_table(
            db_path_str,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
            ),
        )
        .unwrap();

        sqlite::write_results(
            db_path_str,
            &test_results,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::INSERT_TEST_RESULTS_TEMPLATE,
            ),
        )
        .unwrap();

        // Count should be 1
        let count = sqlite::count_rows(
            db_path_str,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::COUNT_TABLE_ROWS_TEMPLATE,
            ),
        )
        .unwrap();
        assert_eq!(count, 1);

        // Delete all rows
        sqlite::delete_all_rows(
            db_path_str,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::DELETE_ALL_ROWS_TEMPLATE,
            ),
        )
        .unwrap();

        // Count should be 0
        let count = sqlite::count_rows(
            db_path_str,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::COUNT_TABLE_ROWS_TEMPLATE,
            ),
        )
        .unwrap();
        assert_eq!(count, 0);
    }
}

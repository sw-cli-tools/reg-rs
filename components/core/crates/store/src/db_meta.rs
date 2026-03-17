use crate::db::with_lock;
use crate::queries;
use crate::queries::statements;
use crate::sqlite;
use crate::sqlite_diff;
use reg_rs_types::error::Result;

/// Count latest test results
pub fn count_latest_results(db_name: &str) -> Result<u32> {
    log::info!("db/count_latest_results {}", &db_name);
    with_lock(db_name, || {
        sqlite_diff::count_rows(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::COUNT_TABLE_ROWS_TEMPLATE,
            ),
        )
    })
}

/// Store test metadata (key-value pair)
pub fn store_metadata(db_name: &str, key: &str, value: &str) -> Result<()> {
    log::info!("db/store_metadata {} key={}", db_name, key);
    with_lock(db_name, || sqlite_diff::store_metadata(db_name, key, value))
}

/// Read test metadata by key. Returns None if not set.
pub fn read_metadata(db_name: &str, key: &str) -> Result<Option<String>> {
    log::info!("db/read_metadata {} key={}", db_name, key);
    with_lock(db_name, || sqlite_diff::read_metadata(db_name, key))
}

/// Clear latest test results
pub fn clear_latest_results(db_name: &str) -> Result<()> {
    log::info!("db/clear_results {}", &db_name);
    with_lock(db_name, || {
        sqlite::delete_all_rows(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::latest(),
                statements::DELETE_ALL_ROWS_TEMPLATE,
            ),
        )
    })
}

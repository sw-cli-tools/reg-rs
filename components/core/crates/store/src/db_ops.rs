use crate::db::with_lock;
use crate::queries;
use crate::queries::statements;
use crate::sqlite;
use crate::sqlite_diff;
use reg_rs_types::error::Result;
use reg_rs_types::types::RegressionType;

// Re-export db_meta functions for backward compatibility
pub use crate::db_meta::{
    clear_latest_results, count_latest_results, read_metadata, store_metadata,
};

/// Reset test result differences
pub fn reset_differences(db_name: &str) -> Result<()> {
    log::info!("db/reset_differences {}", &db_name);
    with_lock(db_name, || {
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
        Ok(())
    })
}

/// Store test result differences
pub fn store_difference(
    db_name: &str,
    difference_type: RegressionType,
    difference_chunk: &str,
) -> Result<()> {
    log::info!("db/store_difference {}", &db_name);
    with_lock(db_name, || {
        let difference_type = (difference_type as usize).to_string();
        sqlite_diff::write_difference(db_name, &difference_type, difference_chunk)?;
        Ok(())
    })
}

/// Read test result differences
pub fn read_differences(db_name: &str) -> Result<Vec<(String, String)>> {
    log::info!("db/read_differences {}", &db_name);
    with_lock(db_name, || {
        sqlite_diff::read_differences(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::differences(),
                statements::SELECT_DIFFERENCES_TEMPLATE,
            ),
        )
    })
}

/// Count test result differences
pub fn count_differences(db_name: &str) -> Result<u32> {
    log::info!("db/count_differences {}", &db_name);
    with_lock(db_name, || {
        sqlite_diff::count_rows(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::differences(),
                statements::COUNT_TABLE_ROWS_TEMPLATE,
            ),
        )
    })
}

/// Count test differences by type
pub fn difference_count_by_type(db_name: &str, difference_type: u8) -> Result<u32> {
    log::info!("db/difference_count_by_type {}", &db_name);
    with_lock(db_name, || {
        sqlite_diff::count_differences_by_type(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::difference_count_by_type(difference_type),
                statements::COUNT_DIFF_TYPE_TEMPLATE,
            ),
        )
    })
}

/// Clear test result differences
pub fn clear_differences(db_name: &str) -> Result<()> {
    log::info!("db/clear_differences {}", &db_name);
    with_lock(db_name, || {
        sqlite::create_table(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::differences(),
                statements::CREATE_DIFFERENCES_TABLE_TEMPLATE,
            ),
        )?;
        sqlite::delete_all_rows(
            db_name,
            &queries::get_statement(
                &queries::StatementContext::differences(),
                statements::DELETE_ALL_ROWS_TEMPLATE,
            ),
        )
    })
}

use rusqlite::Result;

use crate::diff;
use crate::queries;
use crate::runner;
use crate::sqlite;
use crate::templates::statements;

pub(crate) fn store_results(
    db_name: &str,
    test_results: &runner::TestResults,
    statement_context: queries::StatementContext,
) -> Result<()> {
    sqlite::create_table(
        db_name,
        &queries::get_statement(
            &statement_context,
            &statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
        ),
    )?;
    sqlite::write_results(
        db_name,
        test_results,
        &queries::get_statement(
            &statement_context,
            &statements::INSERT_TEST_RESULTS_TEMPLATE,
        ),
    )?;
    Ok(())
}

pub fn read_original_results(db_name: &str) -> Result<runner::TestResults> {
    Ok(sqlite::read_results(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::original(),
            &statements::SELECT_TEST_RESULTS_TEMPLATE,
        ),
    )?)
}

pub fn reset_latest_results(db_name: &str) -> Result<()> {
    sqlite::drop_table(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            &statements::DROP_TABLE_TEMPLATE,
        ),
    )?;
    sqlite::create_table(
        db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            &statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
        ),
    )?;
    Ok(())
}

pub fn drop_all_results(db_name: &str) -> Result<()> {
    sqlite::drop_table(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::original(),
            &statements::DROP_TABLE_TEMPLATE,
        ),
    )?;
    reset_latest_results(&db_name)?;
    Ok(())
}

pub fn reset_differences(db_name: &str) -> Result<()> {
    sqlite::drop_table(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            &statements::DROP_TABLE_TEMPLATE,
        ),
    )?;
    sqlite::create_table(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            &statements::CREATE_DIFFERENCES_TABLE_TEMPLATE,
        ),
    )?;
    Ok(())
}

pub fn store_difference(
    db_name: &str,
    difference_type: diff::RegressionType,
    difference_chunk: &str,
) -> Result<()> {
    let difference_type = (difference_type as usize).to_string();
    sqlite::write_difference(&db_name, &difference_type, &difference_chunk)?;
    Ok(())
}

pub fn read_latest_results(db_name: &str) -> Result<runner::TestResults> {
    Ok(sqlite::read_results(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            &statements::SELECT_TEST_RESULTS_TEMPLATE,
        ),
    )?)
}

pub fn read_differences(db_name: &str) -> Result<Vec<(String, String)>> {
    Ok(sqlite::read_differences(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            &statements::SELECT_DIFFERENCES_TEMPLATE,
        ),
    )?)
}

pub fn count_differences(db_name: &str) -> Result<u32> {
    Ok(sqlite::count_rows(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            &statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )?)
}
pub fn latest_results_table_count(db_name: &str) -> Result<u32> {
    Ok(sqlite::count_rows(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            &statements::TABLE_EXISTS_TEMPLATE,
        ),
    )?)
}

pub fn difference_count_by_type(db_name: &str, difference_type: u8) -> Result<u32> {
    Ok(sqlite::count_differences_by_type(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::difference_count_by_type(difference_type),
            &statements::COUNT_DIFF_TYPE_TEMPLATE,
        ),
    )?)
}

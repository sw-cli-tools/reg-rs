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

pub fn drop_latest_results(db_name: &str) -> Result<()> {
    sqlite::drop_table(
        &db_name,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            &statements::DROP_TABLE_TEMPLATE,
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
    drop_latest_results(&db_name)?;
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

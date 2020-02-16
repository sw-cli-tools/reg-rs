use rusqlite::Result;

use crate::queries;
use crate::runner;
use crate::sqlite;
use crate::templates::statements;

pub(crate) fn store_results(
    db_name: &str,
    test: runner::TestResults,
    statement_context: queries::StatementContext
) -> Result<()> {
    sqlite::create_table(db_name,
                         &queries::get_statement(&statement_context,
                                                 &statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE))?;
    sqlite::write_results(db_name, test,
                          &queries::get_statement(&statement_context,
                                                  &statements::INSERT_TEST_RESULTS_TEMPLATE))?;
    Ok(())
}

pub fn read_original_results(db_name: &str) -> Result<runner::TestResults> {
    Ok(sqlite::read_results(&db_name,
                            &queries::get_statement(&queries::StatementContext::original(),
                                                    &statements::SELECT_TEST_RESULTS_TEMPLATE))?)
}

pub fn drop_latest_results(db_name: &str) -> Result<()> {
    sqlite::drop_table(&db_name,
                 &queries::get_statement(&queries::StatementContext::latest(),
                                         &statements::DROP_TABLE_TEMPLATE))?;
    Ok(())
}

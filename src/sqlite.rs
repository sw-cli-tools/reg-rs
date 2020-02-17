use rusqlite::{params, Connection, Result};

use crate::queries;
use crate::runner::TestResults;
use crate::templates::statements;

pub(crate) fn read_results(db_name: &str, select_statement: &str) -> Result<TestResults> {
    let conn = Connection::open(&db_name)?;
    let mut stmt = conn.prepare(&select_statement)?;
    let mut test_iter = stmt.query_map(params![], |row| {
        Ok(TestResults {
            name: row.get(0)?,
            command: row.get(1)?,
            time_created: row.get(2)?,
            exit_code: row.get(3)?,
            stderr: row.get(4)?,
            stdout: row.get(5)?,
        })
    })?;

    let test = test_iter.next();
    test.unwrap()
}

pub(crate) fn create_table(db_name: &str, create_statement: &str) -> Result<()> {
    let conn = Connection::open(&db_name)?;
    conn.execute(&create_statement, params![])?;
    Ok(())
}

pub(crate) fn write_results(
    db_name: &str,
    test: &TestResults,
    insert_statement: &str,
) -> Result<()> {
    let conn = Connection::open(&db_name)?;
    conn.execute(
        &insert_statement,
        params![
            test.name,
            test.command,
            test.time_created,
            test.exit_code,
            test.stderr,
            test.stdout
        ],
    )?;

    Ok(())
}

pub(crate) fn drop_table(db_name: &str, drop_statement: &str) -> Result<()> {
    let conn = Connection::open(&db_name)?;
    conn.execute(&drop_statement, params![])?;
    Ok(())
}

pub fn write_difference(
    db_name: &str,
    difference_type: &str,
    difference_chunk: &str,
) -> Result<()> {
    let conn = Connection::open(&db_name)?;
    conn.execute(
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::INSERT_DIFFERENCE_TEMPLATE,
        ),
        params![difference_type, difference_chunk,],
    )?;
    Ok(())
}

pub(crate) fn read_differences(db_name: &str, select_statement: &str) -> Result<Vec<(String, String)>> {
    let conn = Connection::open(&db_name)?;
    let mut stmt = conn.prepare(&select_statement)?;
    let difference_iter = stmt.query_map(params![], |row| {
        Ok((row.get(0)?, row.get(1)?,
        ))
    })?;

    let mut result = vec![];
    for difference in difference_iter {
        result.push(difference.unwrap());
    }
    Ok(result)
}

pub fn count_rows(db_name: &str, count_statement: &str) -> Result<u32> {
    let conn = Connection::open(&db_name)?;
    let mut stmt = conn.prepare(&count_statement)?;
    let mut count_iter = stmt.query_map(params![], |row| {
        Ok(row.get(0)?)
    })?;

    let count = count_iter.next();
    count.unwrap()
}

pub fn table_exists(db_name: &str, table_exists_statement: &str) -> Result<u32> {
    let conn = Connection::open(&db_name)?;
    let mut stmt = conn.prepare(&table_exists_statement)?;
    let mut count_iter = stmt.query_map(params![], |row| {
        Ok(row.get(0)?)
    })?;

    let count = count_iter.next();
    count.unwrap()
}

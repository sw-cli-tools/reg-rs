use rusqlite::{Connection, params};

use crate::error::{RegError, Result};
use crate::queries;
use crate::runner::TestResults;
use crate::templates::statements;

pub(crate) fn read_results(db_name: &str, select_statement: &str) -> Result<TestResults> {
    log::info!("sqlite/read_results {} {}", &db_name, &select_statement);
    let test;
    let conn = Connection::open(db_name)?;
    {
        let mut stmt = conn.prepare(select_statement)?;
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
        test = test_iter.next();
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    match test {
        Some(t) => Ok(t?),
        None => Err(RegError::TestNotFound(db_name.to_string())),
    }
}

pub(crate) fn create_table(db_name: &str, create_statement: &str) -> Result<()> {
    log::info!("sqlite/create_table {} {}", &db_name, &create_statement);
    let mut conn = Connection::open(db_name)?;
    {
        let tx = conn.transaction()?;
        tx.execute(create_statement, params![])?;
        tx.commit()?;
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    Ok(())
}

pub(crate) fn write_results(
    db_name: &str,
    test: &TestResults,
    insert_statement: &str,
) -> Result<()> {
    log::info!("sqlite/write_results {} {}", &db_name, &insert_statement);
    let mut conn = Connection::open(db_name)?;
    {
        let tx = conn.transaction()?;
        tx.execute(
            insert_statement,
            params![
                test.name,
                test.command,
                test.time_created,
                test.exit_code,
                test.stderr,
                test.stdout
            ],
        )?;
        tx.commit()?;
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    Ok(())
}

pub(crate) fn drop_table(db_name: &str, drop_statement: &str) -> Result<()> {
    log::info!("sqlite/drop_table {} {}", &db_name, &drop_statement);
    let mut conn = Connection::open(db_name)?;
    {
        let tx = conn.transaction()?;
        tx.execute(drop_statement, params![])?;
        tx.commit()?;
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    Ok(())
}

pub(crate) fn delete_all_rows(db_name: &str, delete_statement: &str) -> Result<()> {
    log::info!("sqlite/remove_rows {} {}", &db_name, &delete_statement);
    let mut conn = Connection::open(db_name)?;
    {
        let tx = conn.transaction()?;
        tx.execute(delete_statement, params![])?;
        tx.commit()?;
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    Ok(())
}

/// write difference to DB
pub fn write_difference(
    db_name: &str,
    difference_type: &str,
    difference_chunk: &str,
) -> Result<()> {
    log::info!("sqlite/write_difference {} {}", &db_name, &difference_type);
    let mut conn = Connection::open(db_name)?;
    {
        let tx = conn.transaction()?;
        tx.execute(
            &queries::get_statement(
                &queries::StatementContext::differences(),
                statements::INSERT_DIFFERENCE_TEMPLATE,
            ),
            params![difference_type, difference_chunk,],
        )?;
        tx.commit()?;
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    Ok(())
}

/// read test result differences
pub(crate) fn read_differences(
    db_name: &str,
    select_statement: &str,
) -> Result<Vec<(String, String)>> {
    log::info!("sqlite/read_differences {} {}", &db_name, &select_statement);
    let mut result = vec![];
    let conn = Connection::open(db_name)?;
    {
        let mut stmt = conn.prepare(select_statement)?;
        let difference_iter = stmt.query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))?;

        for difference in difference_iter {
            result.push(difference?);
        }
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    Ok(result)
}

/// count rows in a table
pub fn count_rows(db_name: &str, count_statement: &str) -> Result<u32> {
    log::info!("sqlite/count_rows {} {}", &db_name, &count_statement);
    let count;
    let conn = Connection::open(db_name)?;
    {
        let mut stmt = conn.prepare(count_statement)?;
        let mut count_iter = stmt.query_map(params![], |row| row.get(0))?;

        count = count_iter.next();
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    match count {
        Some(c) => Ok(c?),
        None => Err(RegError::Database(rusqlite::Error::QueryReturnedNoRows)),
    }
}

/// check that a table exists
pub fn table_exists(db_name: &str, table_exists_statement: &str) -> Result<u32> {
    log::info!(
        "sqlite/table_exists {} {}",
        &db_name,
        &table_exists_statement
    );
    let count;
    let conn = Connection::open(db_name)?;
    {
        let mut stmt = conn.prepare(table_exists_statement)?;
        let mut count_iter = stmt.query_map(params![], |row| row.get(0))?;
        count = count_iter.next();
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    match count {
        Some(c) => Ok(c?),
        None => Err(RegError::Database(rusqlite::Error::QueryReturnedNoRows)),
    }
}

/// Store a metadata key-value pair, creating the table if needed.
pub(crate) fn store_metadata(db_name: &str, key: &str, value: &str) -> Result<()> {
    log::info!("sqlite/store_metadata {} key={}", db_name, key);
    let mut conn = Connection::open(db_name)?;
    {
        let tx = conn.transaction()?;
        tx.execute(statements::CREATE_METADATA_TABLE, params![])?;
        tx.execute(statements::UPSERT_METADATA, params![key, value])?;
        tx.commit()?;
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    Ok(())
}

/// Read a metadata value by key. Returns None if the table or key doesn't exist.
pub(crate) fn read_metadata(db_name: &str, key: &str) -> Result<Option<String>> {
    log::info!("sqlite/read_metadata {} key={}", db_name, key);
    let conn = Connection::open(db_name)?;
    let result = {
        // Table may not exist in older .tdb files
        match conn.prepare(statements::SELECT_METADATA) {
            Ok(mut stmt) => {
                let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
                match rows.next() {
                    Some(val) => Some(val?),
                    None => None,
                }
            }
            Err(_) => None, // metadata_table doesn't exist
        }
    };
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    Ok(result)
}

/// count differences by type
pub fn count_differences_by_type(db_name: &str, count_diff_type_statement: &str) -> Result<u32> {
    log::info!(
        "sqlite/count_differences_by_type {} {}",
        &db_name,
        &count_diff_type_statement
    );
    let count;
    let conn = Connection::open(db_name)?;
    {
        let mut stmt = conn.prepare(count_diff_type_statement)?;
        let mut count_iter = stmt.query_map(params![], |row| row.get(0))?;
        count = count_iter.next();
    }
    conn.close().map_err(|(_, e)| RegError::Database(e))?;
    match count {
        Some(c) => Ok(c?),
        None => Err(RegError::Database(rusqlite::Error::QueryReturnedNoRows)),
    }
}

#[cfg(test)]
#[path = "sqlite_tests.rs"]
mod tests;

use rusqlite::{Connection, params};

use reg_rs_types::error::{RegError, Result};
use reg_rs_types::types::TestResults;

/// Read test results from a table
pub fn read_results(db_name: &str, select_statement: &str) -> Result<TestResults> {
    log::info!("sqlite/read_results {} {}", &db_name, &select_statement);
    let test;
    let conn = Connection::open(db_name).map_err(|e| RegError::Database(e.to_string()))?;
    {
        let mut stmt = conn
            .prepare(select_statement)
            .map_err(|e| RegError::Database(e.to_string()))?;
        let mut test_iter = stmt
            .query_map(params![], |row| {
                Ok(TestResults {
                    name: row.get(0)?,
                    command: row.get(1)?,
                    time_created: row.get(2)?,
                    exit_code: row.get(3)?,
                    stderr: row.get(4)?,
                    stdout: row.get(5)?,
                })
            })
            .map_err(|e| RegError::Database(e.to_string()))?;
        test = test_iter.next();
    }
    conn.close()
        .map_err(|(_, e)| RegError::Database(e.to_string()))?;
    match test {
        Some(t) => Ok(t.map_err(|e| RegError::Database(e.to_string()))?),
        None => Err(RegError::TestNotFound(db_name.to_string())),
    }
}

/// Create a table in the database
pub fn create_table(db_name: &str, create_statement: &str) -> Result<()> {
    log::info!("sqlite/create_table {} {}", &db_name, &create_statement);
    let mut conn = Connection::open(db_name).map_err(|e| RegError::Database(e.to_string()))?;
    {
        let tx = conn
            .transaction()
            .map_err(|e| RegError::Database(e.to_string()))?;
        tx.execute(create_statement, params![])
            .map_err(|e| RegError::Database(e.to_string()))?;
        tx.commit().map_err(|e| RegError::Database(e.to_string()))?;
    }
    conn.close()
        .map_err(|(_, e)| RegError::Database(e.to_string()))?;
    Ok(())
}

/// Write test results to a table
pub fn write_results(db_name: &str, test: &TestResults, insert_statement: &str) -> Result<()> {
    log::info!("sqlite/write_results {} {}", &db_name, &insert_statement);
    let mut conn = Connection::open(db_name).map_err(|e| RegError::Database(e.to_string()))?;
    {
        let tx = conn
            .transaction()
            .map_err(|e| RegError::Database(e.to_string()))?;
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
        )
        .map_err(|e| RegError::Database(e.to_string()))?;
        tx.commit().map_err(|e| RegError::Database(e.to_string()))?;
    }
    conn.close()
        .map_err(|(_, e)| RegError::Database(e.to_string()))?;
    Ok(())
}

/// Drop a table from the database
pub fn drop_table(db_name: &str, drop_statement: &str) -> Result<()> {
    log::info!("sqlite/drop_table {} {}", &db_name, &drop_statement);
    let mut conn = Connection::open(db_name).map_err(|e| RegError::Database(e.to_string()))?;
    {
        let tx = conn
            .transaction()
            .map_err(|e| RegError::Database(e.to_string()))?;
        tx.execute(drop_statement, params![])
            .map_err(|e| RegError::Database(e.to_string()))?;
        tx.commit().map_err(|e| RegError::Database(e.to_string()))?;
    }
    conn.close()
        .map_err(|(_, e)| RegError::Database(e.to_string()))?;
    Ok(())
}

/// Delete all rows from a table
pub fn delete_all_rows(db_name: &str, delete_statement: &str) -> Result<()> {
    log::info!("sqlite/remove_rows {} {}", &db_name, &delete_statement);
    let mut conn = Connection::open(db_name).map_err(|e| RegError::Database(e.to_string()))?;
    {
        let tx = conn
            .transaction()
            .map_err(|e| RegError::Database(e.to_string()))?;
        tx.execute(delete_statement, params![])
            .map_err(|e| RegError::Database(e.to_string()))?;
        tx.commit().map_err(|e| RegError::Database(e.to_string()))?;
    }
    conn.close()
        .map_err(|(_, e)| RegError::Database(e.to_string()))?;
    Ok(())
}

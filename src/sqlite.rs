use crate::queries;
use crate::runner::TestResults;
use rusqlite::{params, Connection, Result};

pub(crate) fn open_query(db_name: &str, _test_name: &str) -> Result<TestResults> {
    // TODO use or eliminate _test_name?
    let conn = Connection::open(&db_name)?;
    let mut stmt = conn 
        .prepare(queries::SELECT_TEST_RESULTS)?;
    let mut test_iter = stmt.query_map(params![], |row| {
        Ok(TestResults {
            id: row.get(0)?,
            name: row.get(1)?,
            command: row.get(2)?,
            time_created: row.get(3)?,
            exit_code: row.get(4)?,
            stderr: row.get(5)?,
            stdout: row.get(6)?,
        })
    })?;

    let test = test_iter.next();
    test.unwrap()
}

pub(crate) fn maybe_create_table(db_name: &str) -> Result<()> {
    let conn = Connection::open(&db_name)?;
    conn.execute(queries::CREATE_TABLE,
        params![],
    )?;
    Ok(())
}

pub(crate) fn write(db_name: &str, test: TestResults) -> Result<()> {
    let conn = Connection::open(&db_name)?;
    conn.execute(queries::INSERT_TEST_RESULTS,
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

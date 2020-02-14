use crate::runner::TestResults;
use rusqlite::{params, Connection, Result};

pub(crate) fn open_query(db_name: &str, _test_name: &str) -> Result<TestResults> {
    // TODO use or eliminate _test_name?
    let conn = Connection::open(&db_name)?;
    let mut stmt = conn 
        .prepare( // TODO change table name to test_results
            "
 SELECT id, name, command, time_created, exit_code, stderr, stdout 
 FROM test
 ORDER BY time_created DESC
",
        )?;
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
    conn.execute( // TODO change table name to test_results
        "CREATE TABLE IF NOT EXISTS test (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  command         TEXT NOT NULL,
                  time_created    TEXT NOT NULL,
                  exit_code       INTEGER,
                  stderr          TEXT NOT NULL,
                  stdout          TEXT NOT NULL
                  )",
        params![],
    )?;
    Ok(())
}

pub(crate) fn write(db_name: &str, test: TestResults) -> Result<()> {
    let conn = Connection::open(&db_name)?;
    conn.execute( // TODO change table name to test_results
        "INSERT INTO test (name, command, time_created, exit_code, stderr, stdout)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
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

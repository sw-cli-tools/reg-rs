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
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_db() -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        (dir, path.to_str().unwrap().to_string())
    }

    fn create_original_table(db: &str) {
        create_table(
            db,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
            ),
        )
        .unwrap();
    }

    fn sample_results() -> TestResults {
        TestResults {
            name: "test1".to_string(),
            command: "echo hello".to_string(),
            time_created: "2024-01-01T12:00:00".to_string(),
            exit_code: 0,
            stderr: "".to_string(),
            stdout: "hello\n".to_string(),
        }
    }

    #[test]
    fn test_create_and_drop_table() {
        let (_dir, db) = test_db();
        create_original_table(&db);
        drop_table(
            &db,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::DROP_TABLE_TEMPLATE,
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_write_and_read_results() {
        let (_dir, db) = test_db();
        create_original_table(&db);
        write_results(
            &db,
            &sample_results(),
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::INSERT_TEST_RESULTS_TEMPLATE,
            ),
        )
        .unwrap();
        let read = read_results(
            &db,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::SELECT_TEST_RESULTS_TEMPLATE,
            ),
        )
        .unwrap();
        assert_eq!(read.name, "test1");
        assert_eq!(read.command, "echo hello");
        assert_eq!(read.exit_code, 0);
        assert_eq!(read.stdout, "hello\n");
        assert_eq!(read.stderr, "");
    }

    #[test]
    fn test_count_rows() {
        let (_dir, db) = test_db();
        create_original_table(&db);
        let count = count_rows(
            &db,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::COUNT_TABLE_ROWS_TEMPLATE,
            ),
        )
        .unwrap();
        assert_eq!(count, 0);

        write_results(
            &db,
            &sample_results(),
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::INSERT_TEST_RESULTS_TEMPLATE,
            ),
        )
        .unwrap();
        let count = count_rows(
            &db,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::COUNT_TABLE_ROWS_TEMPLATE,
            ),
        )
        .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_delete_all_rows() {
        let (_dir, db) = test_db();
        create_original_table(&db);
        write_results(
            &db,
            &sample_results(),
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::INSERT_TEST_RESULTS_TEMPLATE,
            ),
        )
        .unwrap();
        delete_all_rows(
            &db,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::DELETE_ALL_ROWS_TEMPLATE,
            ),
        )
        .unwrap();
        let count = count_rows(
            &db,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::COUNT_TABLE_ROWS_TEMPLATE,
            ),
        )
        .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_write_and_read_differences() {
        let (_dir, db) = test_db();
        create_table(
            &db,
            &queries::get_statement(
                &queries::StatementContext::differences(),
                statements::CREATE_DIFFERENCES_TABLE_TEMPLATE,
            ),
        )
        .unwrap();
        write_difference(&db, "1", "exit code 42").unwrap();
        write_difference(&db, "3", "stderr output").unwrap();
        let diffs = read_differences(
            &db,
            &queries::get_statement(
                &queries::StatementContext::differences(),
                statements::SELECT_DIFFERENCES_TEMPLATE,
            ),
        )
        .unwrap();
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0], ("1".to_string(), "exit code 42".to_string()));
        assert_eq!(diffs[1], ("3".to_string(), "stderr output".to_string()));
    }

    #[test]
    fn test_count_differences_by_type() {
        let (_dir, db) = test_db();
        create_table(
            &db,
            &queries::get_statement(
                &queries::StatementContext::differences(),
                statements::CREATE_DIFFERENCES_TABLE_TEMPLATE,
            ),
        )
        .unwrap();
        write_difference(&db, "1", "code1").unwrap();
        write_difference(&db, "1", "code2").unwrap();
        write_difference(&db, "3", "stderr").unwrap();
        let count = super::count_differences_by_type(
            &db,
            &queries::get_statement(
                &queries::StatementContext::difference_count_by_type(1),
                statements::COUNT_DIFF_TYPE_TEMPLATE,
            ),
        )
        .unwrap();
        assert_eq!(count, 2);
        let count = super::count_differences_by_type(
            &db,
            &queries::get_statement(
                &queries::StatementContext::difference_count_by_type(3),
                statements::COUNT_DIFF_TYPE_TEMPLATE,
            ),
        )
        .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_read_results_empty_table_returns_error() {
        let (_dir, db) = test_db();
        create_original_table(&db);
        let result = read_results(
            &db,
            &queries::get_statement(
                &queries::StatementContext::original(),
                statements::SELECT_TEST_RESULTS_TEMPLATE,
            ),
        );
        assert!(result.is_err());
    }
}

use rusqlite::{Connection, params};

use crate::queries;
use crate::queries::statements;
use reg_rs_types::error::{RegError, Result};

/// Write difference to DB
pub fn write_difference(
    db_name: &str,
    difference_type: &str,
    difference_chunk: &str,
) -> Result<()> {
    log::info!("sqlite/write_difference {} {}", &db_name, &difference_type);
    let mut conn = Connection::open(db_name).map_err(|e| RegError::Database(e.to_string()))?;
    {
        let tx = conn
            .transaction()
            .map_err(|e| RegError::Database(e.to_string()))?;
        tx.execute(
            &queries::get_statement(
                &queries::StatementContext::differences(),
                statements::INSERT_DIFFERENCE_TEMPLATE,
            ),
            params![difference_type, difference_chunk,],
        )
        .map_err(|e| RegError::Database(e.to_string()))?;
        tx.commit().map_err(|e| RegError::Database(e.to_string()))?;
    }
    conn.close()
        .map_err(|(_, e)| RegError::Database(e.to_string()))?;
    Ok(())
}

/// Read test result differences
pub fn read_differences(db_name: &str, select_statement: &str) -> Result<Vec<(String, String)>> {
    log::info!("sqlite/read_differences {} {}", &db_name, &select_statement);
    let mut result = vec![];
    let conn = Connection::open(db_name).map_err(|e| RegError::Database(e.to_string()))?;
    {
        let mut stmt = conn
            .prepare(select_statement)
            .map_err(|e| RegError::Database(e.to_string()))?;
        let difference_iter = stmt
            .query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| RegError::Database(e.to_string()))?;

        for difference in difference_iter {
            result.push(difference.map_err(|e| RegError::Database(e.to_string()))?);
        }
    }
    conn.close()
        .map_err(|(_, e)| RegError::Database(e.to_string()))?;
    Ok(result)
}

/// Count rows in a table
pub fn count_rows(db_name: &str, count_statement: &str) -> Result<u32> {
    log::info!("sqlite/count_rows {} {}", &db_name, &count_statement);
    let count;
    let conn = Connection::open(db_name).map_err(|e| RegError::Database(e.to_string()))?;
    {
        let mut stmt = conn
            .prepare(count_statement)
            .map_err(|e| RegError::Database(e.to_string()))?;
        let mut count_iter = stmt
            .query_map(params![], |row| row.get(0))
            .map_err(|e| RegError::Database(e.to_string()))?;

        count = count_iter.next();
    }
    conn.close()
        .map_err(|(_, e)| RegError::Database(e.to_string()))?;
    match count {
        Some(c) => Ok(c.map_err(|e| RegError::Database(e.to_string()))?),
        None => Err(RegError::Database("Query returned no rows".to_string())),
    }
}

/// Check that a table exists
pub fn table_exists(db_name: &str, table_exists_statement: &str) -> Result<u32> {
    log::info!(
        "sqlite/table_exists {} {}",
        &db_name,
        &table_exists_statement
    );
    count_rows(db_name, table_exists_statement)
}

/// Store a metadata key-value pair, creating the table if needed.
pub fn store_metadata(db_name: &str, key: &str, value: &str) -> Result<()> {
    log::info!("sqlite/store_metadata {} key={}", db_name, key);
    let mut conn = Connection::open(db_name).map_err(|e| RegError::Database(e.to_string()))?;
    {
        let tx = conn
            .transaction()
            .map_err(|e| RegError::Database(e.to_string()))?;
        tx.execute(statements::CREATE_METADATA_TABLE, params![])
            .map_err(|e| RegError::Database(e.to_string()))?;
        tx.execute(statements::UPSERT_METADATA, params![key, value])
            .map_err(|e| RegError::Database(e.to_string()))?;
        tx.commit().map_err(|e| RegError::Database(e.to_string()))?;
    }
    conn.close()
        .map_err(|(_, e)| RegError::Database(e.to_string()))?;
    Ok(())
}

/// Read a metadata value by key. Returns None if the table or key doesn't exist.
pub fn read_metadata(db_name: &str, key: &str) -> Result<Option<String>> {
    log::info!("sqlite/read_metadata {} key={}", db_name, key);
    let conn = Connection::open(db_name).map_err(|e| RegError::Database(e.to_string()))?;
    let result = {
        // Table may not exist in older .tdb files
        match conn.prepare(statements::SELECT_METADATA) {
            Ok(mut stmt) => {
                let mut rows = stmt
                    .query_map(params![key], |row| row.get(0))
                    .map_err(|e| RegError::Database(e.to_string()))?;
                match rows.next() {
                    Some(val) => Some(val.map_err(|e| RegError::Database(e.to_string()))?),
                    None => None,
                }
            }
            Err(_) => None, // metadata_table doesn't exist
        }
    };
    conn.close()
        .map_err(|(_, e)| RegError::Database(e.to_string()))?;
    Ok(result)
}

/// Count differences by type
pub fn count_differences_by_type(db_name: &str, count_diff_type_statement: &str) -> Result<u32> {
    log::info!(
        "sqlite/count_differences_by_type {} {}",
        &db_name,
        &count_diff_type_statement
    );
    count_rows(db_name, count_diff_type_statement)
}

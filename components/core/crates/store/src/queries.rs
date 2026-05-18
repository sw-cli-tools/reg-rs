use serde::Serialize;
use tinytemplate::TinyTemplate;

/// SQL statement template constants
pub mod statements {
    /// create test results table SQL statement template
    pub const CREATE_TEST_RESULTS_TABLE_TEMPLATE: &str = "
 CREATE TABLE IF NOT EXISTS { table_name } (
 name              TEXT NOT NULL,
 command           TEXT NOT NULL,
 time_created      TEXT NOT NULL,
 exit_code         INTEGER,
 stderr            TEXT NOT NULL,
 stdout            TEXT NOT NULL,
 lock              CHAR(1) NOT NULL DEFAULT 'L',
 CONSTRAINT id     PRIMARY KEY (lock),
 CONSTRAINT locked CHECK (lock='L')
)
";

    /// insert test results SQL statement template
    pub const INSERT_TEST_RESULTS_TEMPLATE: &str = "
 INSERT INTO { table_name } (
 name, command, time_created, exit_code, stderr, stdout)
 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
";

    /// select test results SQL statement template
    pub const SELECT_TEST_RESULTS_TEMPLATE: &str = "
 SELECT name, command, time_created, exit_code, stderr, stdout
 FROM { table_name }
";

    /// drop test results table (if exists) SQL statement template
    pub const DROP_TABLE_TEMPLATE: &str = "
 DROP TABLE IF EXISTS { table_name }
";

    /// delete all rows from a table SQL statement template
    pub const DELETE_ALL_ROWS_TEMPLATE: &str = "
 DELETE FROM { table_name }
";

    /// create differences table SQL statement template
    pub const CREATE_DIFFERENCES_TABLE_TEMPLATE: &str = "
 CREATE TABLE IF NOT EXISTS { table_name } (
 id                INTEGER PRIMARY KEY,
 type              TEXT NOT NULL,
 chunk             TEXT
)
";

    /// insert test difference SQL statement template
    pub const INSERT_DIFFERENCE_TEMPLATE: &str = "
 INSERT INTO { table_name } (type, chunk)
 VALUES (?1, ?2)
";

    /// select test differences SQL statement template
    pub const SELECT_DIFFERENCES_TEMPLATE: &str = "
 SELECT type, chunk
 FROM { table_name }
";

    /// count table rows SQL statement template
    pub const COUNT_TABLE_ROWS_TEMPLATE: &str = "
SELECT COUNT(*) AS c from { table_name }
";

    /// count differences by type SQL statement template
    pub const COUNT_DIFF_TYPE_TEMPLATE: &str = "
SELECT COUNT(*) FROM { table_name } WHERE type = '{ difference_type }'
";

    /// create metadata table SQL statement (not templated — fixed table name)
    pub const CREATE_METADATA_TABLE: &str = "
 CREATE TABLE IF NOT EXISTS metadata_table (
 key               TEXT PRIMARY KEY,
 value             TEXT NOT NULL
)
";

    /// upsert metadata SQL statement
    pub const UPSERT_METADATA: &str = "
 INSERT INTO metadata_table (key, value) VALUES (?1, ?2)
 ON CONFLICT(key) DO UPDATE SET value = excluded.value
";

    /// select metadata value by key SQL statement
    pub const SELECT_METADATA: &str = "
 SELECT value FROM metadata_table WHERE key = ?1
";
}

/// Specifies which DB table to use
#[derive(Debug, Serialize)]
pub struct StatementContext {
    difference_type: u8,
    table_name: String,
}

impl StatementContext {
    /// original results table
    pub fn original() -> Self {
        StatementContext {
            difference_type: 0,
            table_name: "original_results_table".to_string(),
        }
    }
    /// latest results table
    pub fn latest() -> Self {
        StatementContext {
            difference_type: 0,
            table_name: "latest_results_table".to_string(),
        }
    }
    /// latest differences table
    pub fn differences() -> Self {
        StatementContext {
            difference_type: 0,
            table_name: "differences_table".to_string(),
        }
    }
    /// difference count by type
    pub fn difference_count_by_type(difference_type: u8) -> Self {
        StatementContext {
            difference_type,
            table_name: "differences_table".to_string(),
        }
    }
}

/// Build a SQL statement by rendering a template with the given context.
pub fn get_statement(statement_context: &StatementContext, statement_template: &str) -> String {
    render(statement_context, statement_template).unwrap_or_else(|e| {
        panic!(
            "SQL template rendering failed (this is a bug): template='{statement_template}', error={e}"
        )
    })
}

/// Fill in template, rendering it only once
fn render(
    statement_context: &StatementContext,
    statement_template: &str,
) -> reg_rs_types::error::Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template("statement_template", statement_template)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    let result = tt
        .render("statement_template", &statement_context)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    log::debug!("render_statement: {}", &result);
    Ok(result)
}

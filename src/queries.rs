use serde::Serialize;
use tinytemplate::TinyTemplate;

pub use crate::templates::statements;

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
///
/// Returns the rendered SQL string, or panics with a descriptive message
/// if the template is malformed (indicates a programming error).
pub fn get_statement(statement_context: &StatementContext, statement_template: &str) -> String {
    render(statement_context, statement_template).unwrap_or_else(|e| {
        panic!(
            "SQL template rendering failed (this is a bug): template='{}', error={}",
            statement_template, e
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_statement_original_table() {
        let stmt = get_statement(
            &StatementContext::original(),
            statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
        );
        assert!(stmt.contains("original_results_table"));
    }

    #[test]
    fn test_get_statement_latest_table() {
        let stmt = get_statement(
            &StatementContext::latest(),
            statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
        );
        assert!(stmt.contains("latest_results_table"));
    }

    #[test]
    fn test_get_statement_differences_table() {
        let stmt = get_statement(
            &StatementContext::differences(),
            statements::CREATE_DIFFERENCES_TABLE_TEMPLATE,
        );
        assert!(stmt.contains("differences_table"));
    }

    #[test]
    fn test_get_statement_drop_table() {
        let stmt = get_statement(
            &StatementContext::original(),
            statements::DROP_TABLE_TEMPLATE,
        );
        assert!(stmt.contains("DROP TABLE IF EXISTS"));
        assert!(stmt.contains("original_results_table"));
    }

    #[test]
    fn test_get_statement_count_diff_type() {
        let stmt = get_statement(
            &StatementContext::difference_count_by_type(3),
            statements::COUNT_DIFF_TYPE_TEMPLATE,
        );
        assert!(stmt.contains("differences_table"));
        assert!(stmt.contains("3"));
    }

    #[test]
    fn test_get_statement_insert() {
        let stmt = get_statement(
            &StatementContext::latest(),
            statements::INSERT_TEST_RESULTS_TEMPLATE,
        );
        assert!(stmt.contains("INSERT INTO latest_results_table"));
    }

    #[test]
    fn test_get_statement_select() {
        let stmt = get_statement(
            &StatementContext::original(),
            statements::SELECT_TEST_RESULTS_TEMPLATE,
        );
        assert!(stmt.contains("SELECT"));
        assert!(stmt.contains("original_results_table"));
    }
}

/// Fill in template, rendering it only once
fn render(
    statement_context: &StatementContext,
    statement_template: &str,
) -> crate::error::Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template("statement_template", statement_template)?;
    let result = tt.render("statement_template", &statement_context)?;
    log::debug!("render_statement: {}", &result);
    Ok(result)
}

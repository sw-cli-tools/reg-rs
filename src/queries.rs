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

/// Fill in template, rendering it only once
fn render(
    statement_context: &StatementContext,
    statement_template: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut tt = TinyTemplate::new();
    tt.add_template("statement_template", statement_template)?;
    let result = tt.render("statement_template", &statement_context)?;
    log::debug!("render_statement: {}", &result);
    Ok(result)
}

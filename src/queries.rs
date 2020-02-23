use tinytemplate::TinyTemplate;

pub use crate::templates::statements;

#[derive(Debug, Serialize)]
pub struct StatementContext {
    difference_type: u8,
    table_name: String,
}

impl StatementContext {
    pub fn original() -> Self {
        StatementContext {
            difference_type: 0,
            table_name: "original_results_table".to_string(),
        }
    }
    pub fn latest() -> Self {
        StatementContext {
            difference_type: 0,
            table_name: "latest_results_table".to_string(),
        }
    }
    pub fn differences() -> Self {
        StatementContext {
            difference_type: 0,
            table_name: "differences_table".to_string(),
        }
    }
    pub fn difference_count_by_type(difference_type: u8) -> Self {
        StatementContext {
            difference_type,
            table_name: "differences_table".to_string(),
        }
    }
}

pub fn get_statement(statement_context: &StatementContext, statement_template: &str) -> String {
    render(statement_context, statement_template).unwrap()
}

fn render(
    statement_context: &StatementContext,
    statement_template: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut tt = TinyTemplate::new();
    tt.add_template("statement_template", statement_template)?;
    md!(tt.render("statement_template", &statement_context)?);
    Ok(tt.render("statement_template", &statement_context)?)
}

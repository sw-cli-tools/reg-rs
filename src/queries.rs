use tinytemplate::TinyTemplate;

pub use crate::templates::statements;

#[derive(Serialize)]
pub struct StatementContext {
    table_name: String,
}

impl StatementContext {
    pub fn original() -> Self {
        StatementContext {
            table_name: "original_results_table".to_string(),
        }
    }
    pub fn latest() -> Self {
        StatementContext {
            table_name: "latest_results_table".to_string(),
        }
    }
    pub fn differences() -> Self {
        StatementContext {
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
    Ok(tt.render("statement_template", &statement_context)?)
}

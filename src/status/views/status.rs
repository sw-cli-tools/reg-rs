use tinytemplate::TinyTemplate;

use crate::templates::views;

#[derive(Serialize)]
pub struct StatusViewContext {
    fail_count: String,
    not_run_count: String,
    pass_count: String,
    server_started: String,
    test_count: String,
    test_pattern: String,
}

impl StatusViewContext {
    pub fn new(
        fail_count: u32,
        not_run_count: u32,
        pass_count: u32,
        server_started: String,
        test_count: u32,
        test_pattern: String,
    ) -> Self {
        StatusViewContext {
            fail_count: format!("{}", fail_count).to_string(), // TODO CSS
            not_run_count: format!("{}", not_run_count).to_string(),
            pass_count: format!("{}", pass_count).to_string(),
            server_started,
            test_count: format!("{}", test_count).to_string(),
            test_pattern,
        }
    }
}

pub fn render(status_view_context: &StatusViewContext
) -> Result<String, Box<dyn std::error::Error>> {
    let mut tt = TinyTemplate::new();
    tt.add_template("status_view_template", views::STATUS_VIEW_TEMPLATE)?;
    let rendered = tt.render("status_view_template", &status_view_context)?;
    Ok(rendered)
}

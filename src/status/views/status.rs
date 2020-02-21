use tinytemplate::TinyTemplate;

use crate::templates::views;

#[derive(Serialize)]
pub struct StatusViewContext {
    fail_count: String,
    failed_test_names: Vec<String>,
    fail_symbol: String,
    no_failed_tests: bool,
    no_not_yet_run_tests: bool,
    no_passed_tests: bool,
    not_run_count: String,
    not_yet_run_test_names: Vec<String>,
    pass_count: String,
    passed_test_names: Vec<String>,
    pass_symbol: String,
    server_started: String,
    test_count: String,
    test_pattern: String,
    warn_symbol: String,
}

impl StatusViewContext {
    pub fn new(
        fail_count: u32,
        failed_test_names: Vec<String>,
        no_failed_tests: bool,
        no_not_yet_run_tests: bool,
        no_passed_tests: bool,        
        not_run_count: u32,
        not_yet_run_test_names: Vec<String>,
        pass_count: u32,
        passed_test_names: Vec<String>,
        server_started: String,
        test_count: u32,
        test_pattern: String,
    ) -> Self {
        StatusViewContext {
            fail_count: format!(" {:05}", fail_count).to_string(), // TODO CSS
            failed_test_names,
            fail_symbol: "-".to_string(),
            no_failed_tests,
            no_not_yet_run_tests,
            no_passed_tests,
            not_run_count: format!(" {:05}", not_run_count).to_string(),
            not_yet_run_test_names,
            pass_count: format!(" {:05}", pass_count).to_string(),
            passed_test_names,
            pass_symbol: "+".to_string(),
            server_started,
            test_count: format!(" {:05}", test_count).to_string(),
            test_pattern,
            warn_symbol: "?".to_string(),
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

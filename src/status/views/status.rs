use log;
use tinytemplate::TinyTemplate;

use crate::status::server;
use crate::templates::views;

#[derive(Serialize)]
pub struct StatusViewContext {
    fail_count: String,
    fail_symbol: String,
    no_failed_tests: bool,
    no_not_yet_run_tests: bool,
    no_passed_tests: bool,
    not_run_count: String,
    pass_count: String,
    pass_symbol: String,
    server_started: String,
    state_updated: String,
    test_count: String,
    test_pattern: String,
    test_runs: Vec<server::TestDetails>,
    warn_symbol: String,
}

impl StatusViewContext {
    pub fn new(
        fail_count: u32,
        no_failed_tests: bool,
        no_not_yet_run_tests: bool,
        no_passed_tests: bool,        
        not_run_count: u32,
        pass_count: u32,
        server_started: String,
        state_updated: String,
        test_count: u32,
        test_pattern: String,
        test_runs: Vec<server::TestDetails>,
    ) -> Self {
        StatusViewContext {
            fail_count: format!(" {:05}", fail_count).to_string(), // TODO CSS
            fail_symbol: "-".to_string(),
            no_failed_tests,
            no_not_yet_run_tests,
            no_passed_tests,
            not_run_count: format!(" {:05}", not_run_count).to_string(),
            pass_count: format!(" {:05}", pass_count).to_string(),
            pass_symbol: "+".to_string(),
            server_started,
            state_updated,
            test_count: format!(" {:05}", test_count).to_string(),
            test_pattern,
            test_runs,
            warn_symbol: "?".to_string(),
        }
    }
}

pub fn render(status_view_context: &StatusViewContext
) -> Result<String, Box<dyn std::error::Error>> {
    log::info!("status/render");
    let mut tt = TinyTemplate::new();
    tt.add_template("status_view_template", views::STATUS_VIEW_TEMPLATE)?;
    let rendered = tt.render("status_view_template", &status_view_context)?;
    Ok(rendered)
}

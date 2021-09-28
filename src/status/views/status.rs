use tinytemplate::TinyTemplate;

use crate::status::server;
use crate::templates::views;

#[derive(Serialize)]
pub struct StatusCounts {
    pub fail_count: String,
    pub not_run_count: String,
    pub pass_count: String,
    pub test_count: String,
}

#[derive(Serialize)]
pub struct StatusFlags {
    pub no_failed_tests: bool,
    pub no_not_yet_run_tests: bool,
    pub no_passed_tests: bool,
}

#[derive(Serialize)]
pub struct StatusViewContext {
    fail_symbol: String,
    pass_symbol: String,
    server_started: String,
    state_updated: String,
    status_counts: StatusCounts,
    status_flags: StatusFlags,
    test_pattern: String,
    test_runs: Vec<server::TestDetails>,
    warn_symbol: String,
}

impl StatusViewContext {
    pub fn new(
        server_started: String,
        state_updated: String,
        status_counts: StatusCounts,
        status_flags: StatusFlags,
        test_pattern: String,
        test_runs: Vec<server::TestDetails>,
    ) -> Self {
        StatusViewContext {
            fail_symbol: "-".to_string(),
            pass_symbol: "+".to_string(),
            server_started,
            state_updated,
            status_counts,
            status_flags,
            test_pattern,
            test_runs,
            warn_symbol: "?".to_string(),
        }
    }
}

pub fn render(
    status_view_context: &StatusViewContext,
) -> Result<String, Box<dyn std::error::Error>> {
    log::info!("status/render");
    let mut tt = TinyTemplate::new();
    tt.add_template("status_view_template", views::STATUS_VIEW_TEMPLATE)?;
    let rendered = tt.render("status_view_template", &status_view_context)?;
    Ok(rendered)
}

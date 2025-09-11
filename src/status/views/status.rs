use tinytemplate::TinyTemplate;

use crate::status::server as server;
use crate::templates::views;

/// Status Counts Data
#[derive(Serialize)]
pub struct StatusCounts {
    /// Fail count
    pub fail_count: String,
    /// Not-run count
    pub not_run_count: String,
    /// Pass count
    pub pass_count: String,
    /// Test count
    pub test_count: String,
}

/// Status flags
#[derive(Serialize)]
pub struct StatusFlags {
    /// No failed tests
    pub no_failed_tests: bool,
    /// No not-yet-run tests
    pub no_not_yet_run_tests: bool,
    /// No passed tests
    pub no_passed_tests: bool,
}

/// Status view template data
#[derive(Serialize)]
pub struct StatusViewContext<'a> {
    fail_symbol: String,
    pass_symbol: String,
    server_started: String,
    state_updated: String,
    status_counts: StatusCounts,
    status_flags: StatusFlags,
    test_pattern: String,
    test_runs: &'a [server::TestDetails],
    warn_symbol: String,
}

impl<'a> StatusViewContext<'a> {
    /// build Status view template data
    pub fn new(
        server_started: String,
        state_updated: String,
        status_counts: StatusCounts,
        status_flags: StatusFlags,
        test_pattern: String,
        test_runs: &'a [server::TestDetails],
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

/// Render status view template
pub fn render(
    status_view_context: &StatusViewContext,
) -> Result<String, Box<dyn std::error::Error>> {
    log::info!("status/render");
    let mut tt = TinyTemplate::new();
    tt.add_template("status_view_template", views::STATUS_VIEW_TEMPLATE)?;
    let rendered = tt.render("status_view_template", status_view_context)?;
    Ok(rendered)
}
use crate::status::TestDetails;
use serde::Serialize;

/// Status view template data
#[derive(Serialize)]
pub struct StatusViewContext<'a> {
    /// Server started timestamp
    pub server_started: String,
    /// Last state update timestamp
    pub state_updated: String,
    /// Status counts (fail, pass, etc)
    pub status_counts: StatusCounts,
    /// Status flags (no failed, etc)
    pub status_flags: StatusFlags,
    /// Test name pattern
    pub test_pattern: String,
    /// List of test runs
    pub test_runs: &'a [TestDetails],
}

impl<'a> StatusViewContext<'a> {
    /// build Status view template data
    pub fn new(
        server_started: String,
        state_updated: String,
        status_counts: StatusCounts,
        status_flags: StatusFlags,
        test_pattern: String,
        test_runs: &'a [TestDetails],
    ) -> Self {
        StatusViewContext {
            server_started,
            state_updated,
            status_counts,
            status_flags,
            test_pattern,
            test_runs,
        }
    }
}

/// Status counts for the view
#[derive(Serialize)]
pub struct StatusCounts {
    /// Number of failed tests
    pub fail_count: String,
    /// Number of tests not yet run
    pub not_run_count: String,
    /// Number of passed tests
    pub pass_count: String,
    /// Total number of tests
    pub test_count: String,
}

/// Status flags for the view
#[derive(Serialize)]
pub struct StatusFlags {
    /// True if no tests failed
    pub no_failed_tests: bool,
    /// True if all tests have been run
    pub no_not_yet_run_tests: bool,
    /// True if no tests passed
    pub no_passed_tests: bool,
}

/// Render status view template
pub fn render(status_view_context: &StatusViewContext) -> crate::error::Result<String> {
    use crate::templates::views;
    log::info!("status/render");
    let mut tt = tinytemplate::TinyTemplate::new();
    tt.add_template("status_view_template", views::STATUS_VIEW_TEMPLATE)?;
    let rendered = tt.render("status_view_template", status_view_context)?;
    Ok(rendered)
}

use serde::Serialize;

use crate::assets;

/// Details about a single test run (used in templates)
#[derive(Debug, Serialize, Clone)]
pub struct TestDetails {
    /// When test was first run
    pub created: String,
    /// Test differences from last run (if any)
    pub diffs: Option<Vec<String>>,
    /// When test was last run (if more than once)
    pub last_ran: Option<String>,
    /// Test name
    pub name: String,
}

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
    /// Build Status view template data
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

/// Generate HTML status indicator for landing page
pub fn status_indicator(fail: usize, pending: usize, total: usize, pass: usize) -> String {
    if fail > 0 {
        format!(
            r#"<span class="status-indicator fail">&#10007; {} failed</span>"#,
            fail
        )
    } else if total == 0 {
        r#"<span class="status-indicator pending">No tests found</span>"#.to_string()
    } else if pending > 0 {
        format!(
            r#"<span class="status-indicator pending">? {} not yet run</span>"#,
            pending
        )
    } else {
        format!(
            r#"<span class="status-indicator pass">&#10003; All {} tests passing</span>"#,
            pass
        )
    }
}

/// HTML template for the landing page, with placeholders for css, status,
/// pattern, pass, fail, pending, total, started, and script.
const LANDING_HTML: &str = r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>reg-rs</title>
<style>{css}</style>
</head><body>
<div id="sse-badge" style="position:fixed;bottom:12px;right:12px;z-index:9999;background:#16a34a;color:#fff;font-family:ui-monospace,monospace;font-size:0.85em;font-weight:600;padding:6px 12px;border-radius:6px;box-shadow:0 2px 6px rgba(0,0,0,0.12);opacity:0.9;">SSE: 0</div>
<div class="container">
  <h1>reg-rs</h1>
  <div class="meta">pattern: <code>{pattern}</code> &nbsp; data: <code>{data_dir}</code> &nbsp; <span id="current-test"></span></div>
  <div class="summary">
    <div id="status-line">{status}</div>
    <div class="counts">
      <span id="c-pass">{pass} passed</span>
      <span id="c-fail">{fail} failed</span>
      <span id="c-pending">{pending} pending</span>
      <span id="c-total">{total} total</span>
    </div>
  </div>
  <ul class="views">
    <li><a href="/status">
      <div class="view-title">Status Dashboard</div>
      <div class="view-desc">Pass/fail details, collapsible sections, diffs for failures</div>
    </a></li>
  </ul>
  <footer>started {started}</footer>
</div>
<script>{script}</script>
</body></html>"##;

/// Generate HTML for landing page
pub fn landing_page(
    pattern: &str,
    server_started: &str,
    fail: usize,
    pass: usize,
    pending: usize,
    total: usize,
    data_dir: &str,
) -> String {
    let status = status_indicator(fail, pending, total, pass);
    LANDING_HTML
        .replace("{css}", assets::LANDING_CSS)
        .replace("{status}", &status)
        .replace("{pattern}", pattern)
        .replace("{data_dir}", data_dir)
        .replace("{pass}", &pass.to_string())
        .replace("{fail}", &fail.to_string())
        .replace("{pending}", &pending.to_string())
        .replace("{total}", &total.to_string())
        .replace("{started}", server_started)
        .replace("{script}", assets::LANDING_SCRIPT)
}

/// Generate HTML for status dashboard page
pub fn status_page(_pattern: &str, server_started: &str, status_view_html: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>reg-rs: status</title>
<style>{}</style>
</head><body>
<div class="container">
  {}
  <footer>
    started {} &middot; <a href="/">Back to Home</a>
  </footer>
</div>
<script>{}</script>
</body></html>"##,
        assets::STATUS_CSS,
        status_view_html,
        server_started,
        assets::STATUS_SCRIPT
    )
}

/// Render status view template body content
pub fn render_status_view(
    status_view_context: &StatusViewContext,
) -> reg_rs_types::error::Result<String> {
    log::info!("status/render");
    let mut tt = tinytemplate::TinyTemplate::new();
    tt.set_default_formatter(&tinytemplate::format_unescaped);
    tt.add_template("status_view_template", assets::STATUS_VIEW_TEMPLATE)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    let rendered = tt
        .render("status_view_template", status_view_context)
        .map_err(|e| reg_rs_types::error::RegError::Template(e.to_string()))?;
    Ok(rendered)
}

/// Categorize test runs into failed, passed, and not-yet-run lists
pub fn categorize_runs(runs: &[TestDetails]) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut failed = vec![];
    let mut not_yet_run = vec![];
    let mut passed = vec![];
    for run in runs {
        if run.last_ran.is_none() {
            not_yet_run.push(run.name.clone());
        } else if run.diffs.is_none() {
            passed.push(run.name.clone());
        } else {
            failed.push(run.name.clone());
        }
    }
    (failed, passed, not_yet_run)
}

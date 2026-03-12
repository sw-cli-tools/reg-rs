use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use serde::Serialize;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::config;
use crate::db;
use crate::diff::RegressionType;
use crate::error::RegError;
use crate::finder;
use crate::status::monitor;
use crate::status::views::status;
use crate::time;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Shared state data
    pub state_data: Arc<Mutex<StateData>>,
}

impl AppState {
    fn new(pattern: String) -> Self {
        let state_data = Arc::new(Mutex::new(StateData::new(pattern.clone())));
        Self { state_data }
    }
}

/// Shared State Data
#[derive(Debug)]
pub struct StateData {
    /// Test name pattern
    pub pattern: String,
    /// List of test results
    pub runs: Vec<TestDetails>,
    server_started: String,
    /// updated time
    pub state_updated: String,
    /// Data directory to watch for changes
    pub data_dir: PathBuf,
}

impl StateData {
    fn new(pattern: String) -> Self {
        let data_dir = crate::data_dir();
        Self {
            pattern,
            runs: vec![],
            server_started: time::now(),
            state_updated: "".to_string(),
            data_dir,
        }
    }
}

/// Test details
#[derive(Debug, Serialize, Clone)]
pub struct TestDetails {
    /// When test was first run
    pub created: String,
    /// Test results differences from last run (if any)
    pub diffs: Option<Vec<String>>,
    /// When test was last run (if more than once)
    pub last_ran: Option<String>,
    /// Test name
    pub name: String,
}

/// start status server
pub(crate) async fn start(config: &config::Config) -> crate::error::Result<()> {
    log::info!("server/start - BEGIN");
    let status_port = config.status_port();
    let pattern = config.extract_pattern().to_string();
    log::info!("server/start - port={}, pattern={}", status_port, pattern);

    log::info!("server/start - creating AppState");
    let app_state = AppState::new(pattern.clone());

    log::info!("server/start - calling initial set_test_runs");
    set_test_runs(app_state.clone())?;
    log::info!("server/start - initial set_test_runs completed");

    log::info!("server/start - launching monitor thread");
    let _handles = monitor::launch_monitor(app_state.clone());
    log::info!("server/start - monitor thread launched");

    let addr = SocketAddr::from(([127, 0, 0, 1], status_port));
    println!("Listening at {}.  Ctrl-C to terminate server", addr);

    log::info!("server/start - creating router");
    let app = Router::new()
        .route("/", get(serve_landing))
        .route("/status", get(serve_status_view))
        .with_state(app_state);

    log::info!("server/start - binding TCP listener to {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("server/start - TCP listener bound, starting axum::serve");

    axum::serve(listener, app.into_make_service()).await?;

    log::info!("server/start - END (server stopped)");
    Ok(())
}

/// Serve the landing page with links to available views
async fn serve_landing(State(state): State<AppState>) -> impl IntoResponse {
    let pattern = match state.state_data.lock() {
        Ok(guard) => guard.pattern.clone(),
        Err(_) => "unknown".to_string(),
    };
    let html = format!(
        r#"<!DOCTYPE html>
<html><head><title>reg-rs</title>
<style>
  body {{ font-family: system-ui, sans-serif; max-width: 600px; margin: 40px auto; padding: 0 20px; }}
  h1 {{ border-bottom: 1px solid #ccc; padding-bottom: 10px; }}
  ul {{ list-style: none; padding: 0; }}
  li {{ margin: 12px 0; }}
  a {{ font-size: 1.2em; }}
  .meta {{ color: #666; font-size: 0.9em; }}
</style>
</head><body>
<h1>reg-rs</h1>
<p class="meta">pattern: <code>{pattern}</code></p>
<ul>
  <li><a href="/status">Status Dashboard</a> — pass/fail summary, diffs, test details</li>
</ul>
</body></html>"#
    );
    (StatusCode::OK, Html(html))
}

/// Serve the status view
async fn serve_status_view(State(state): State<AppState>) -> impl IntoResponse {
    log::info!("server/serve_status_view - START");

    // Update the state BEFORE locking (set_test_runs acquires its own lock)
    log::info!("server/serve_status_view - calling set_test_runs");
    if let Err(e) = set_test_runs(state.clone()) {
        log::error!("Failed to update test runs: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Error updating test runs</h1>".to_string()),
        );
    }
    log::info!("server/serve_status_view - set_test_runs completed");

    // Now lock to read the updated state
    log::info!("server/serve_status_view - acquiring lock");
    let state_data = match state.state_data.lock() {
        Ok(guard) => {
            log::info!("server/serve_status_view - lock acquired");
            guard
        }
        Err(e) => {
            log::error!("Failed to lock state data: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Error: mutex poisoned</h1>".to_string()),
            );
        }
    };

    let status_view = match build_status_view(&state_data) {
        Ok(view) => view,
        Err(e) => {
            log::error!("Failed to render status view: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Error</h1>".to_string()),
            );
        }
    };
    let page = wrap_in_page(&state_data.pattern, &state_data.server_started, &status_view);
    log::info!(
        "server/serve_status_view - END, response len={}",
        page.len()
    );
    (StatusCode::OK, Html(page))
}

/// Build the status view HTML from current state data
fn build_status_view(
    state_data: &std::sync::MutexGuard<'_, StateData>,
) -> crate::error::Result<String> {
    let (failed_test_names, passed_test_names, not_yet_run_test_names) =
        categorize_runs(&state_data.runs);

    let status_counts = status::StatusCounts {
        fail_count: format!(" {:05}", failed_test_names.len()),
        not_run_count: format!(" {:05}", not_yet_run_test_names.len()),
        pass_count: format!(" {:05}", passed_test_names.len()),
        test_count: format!(" {:05}", state_data.runs.len()),
    };
    let status_flags = status::StatusFlags {
        no_failed_tests: failed_test_names.is_empty(),
        no_not_yet_run_tests: not_yet_run_test_names.is_empty(),
        no_passed_tests: passed_test_names.is_empty(),
    };

    status::render(&status::StatusViewContext::new(
        state_data.server_started.clone(),
        state_data.state_updated.clone(),
        status_counts,
        status_flags,
        state_data.pattern.to_string(),
        &state_data.runs,
    ))
}

/// update shared state with test run data
pub(crate) fn set_test_runs(app_state: AppState) -> crate::error::Result<()> {
    log::info!("server/set_test_runs - acquiring lock");
    let mut state_data = app_state
        .state_data
        .lock()
        .map_err(|e| RegError::MutexPoisoned(format!("state_data lock failed: {}", e)))?;
    log::info!(
        "server/set_test_runs - lock acquired, pattern: {}",
        &state_data.pattern
    );
    let mut test_runs = vec![];
    let test_names = finder::discover(state_data.pattern.to_string())?;
    for test_name in &test_names.found {
        let original_result = db::read_original_results(test_name)?;
        let latest_results_row_count = db::count_latest_results(test_name)?;
        let (last_ran, diffs) = if latest_results_row_count == 0 {
            (None, None)
        } else {
            let latest_result = db::read_latest_results(test_name)?;
            let difference_count = db::count_differences(test_name)?;
            let diffs = if difference_count > 0 {
                Some(get_diffs(test_name)?)
            } else {
                None
            };
            (Some(latest_result.time_created), diffs)
        };
        test_runs.push(TestDetails {
            created: original_result.time_created,
            diffs,
            name: test_name.to_string(),
            last_ran,
        });
    }
    state_data.runs = test_runs;
    state_data.state_updated = time::now();
    log::info!(
        "server/set_test_runs - completed, {} tests loaded, releasing lock",
        state_data.runs.len()
    );
    Ok(())
}

/// Format a single difference tuple into an HTML string with diff styling.
/// Returns None for "same" types and unknown codes.
fn format_difference(type_code: &str, value: &str) -> Option<String> {
    let escaped = html_escape(value);
    match RegressionType::from_code(type_code)? {
        RegressionType::ActualCode => {
            Some(format!(r#"<div class="diff-add">+ Actual code: {}</div>"#, escaped))
        }
        RegressionType::ExpectedCode => {
            Some(format!(r#"<div class="diff-remove">- Expected code: {}</div>"#, escaped))
        }
        RegressionType::StderrAdd => {
            Some(format!(r#"<div class="diff-add">+ Stderr add: {}</div>"#, escaped))
        }
        RegressionType::StderrRemove => {
            Some(format!(r#"<div class="diff-remove">- Stderr remove: {}</div>"#, escaped))
        }
        RegressionType::StdoutAdd => {
            Some(format!(r#"<div class="diff-add">+ Stdout add: {}</div>"#, escaped))
        }
        RegressionType::StdoutRemove => {
            Some(format!(r#"<div class="diff-remove">- Stdout remove: {}</div>"#, escaped))
        }
        RegressionType::StderrSame | RegressionType::StdoutSame => None,
    }
}

/// Wrap rendered template body in a full HTML page with CSS
fn wrap_in_page(pattern: &str, server_started: &str, body: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>reg-rs — {pattern}</title>
<style>
  :root {{
    --pass: #16a34a; --fail: #dc2626; --warn: #d97706; --muted: #6b7280;
    --bg: #f9fafb; --card: #fff; --border: #e5e7eb;
    --font: system-ui, -apple-system, sans-serif;
    --mono: ui-monospace, "SF Mono", Menlo, monospace;
  }}
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ font-family: var(--font); background: var(--bg); color: #111; padding: 20px; }}
  .container {{ max-width: 960px; margin: 0 auto; }}
  header {{ display: flex; align-items: baseline; gap: 16px; margin-bottom: 20px;
            border-bottom: 2px solid var(--border); padding-bottom: 12px; }}
  header h1 {{ font-size: 1.4em; }}
  .meta {{ color: var(--muted); font-size: 0.85em; }}
  .meta code {{ font-family: var(--mono); background: #f3f4f6; padding: 1px 5px;
               border-radius: 3px; }}
  nav {{ display: flex; gap: 12px; margin-bottom: 20px; font-size: 0.9em; }}
  nav a {{ color: var(--muted); text-decoration: none; padding: 4px 8px; border-radius: 4px; }}
  nav a:hover {{ background: var(--border); color: #111; }}
  .overview {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
               gap: 12px; margin-bottom: 24px; }}
  .stat {{ background: var(--card); border: 1px solid var(--border); border-radius: 8px;
           padding: 16px; text-align: center; }}
  .stat .number {{ font-size: 2em; font-weight: 700; font-family: var(--mono); }}
  .stat .label {{ font-size: 0.85em; color: var(--muted); margin-top: 4px; }}
  .stat.pass {{ border-left: 4px solid var(--pass); }}
  .stat.fail {{ border-left: 4px solid var(--fail); }}
  .stat.pending {{ border-left: 4px solid var(--warn); }}
  .stat.total {{ border-left: 4px solid var(--border); }}
  .section {{ background: var(--card); border: 1px solid var(--border); border-radius: 8px;
              margin-bottom: 16px; }}
  .section-header {{ padding: 12px 16px; cursor: pointer; display: flex;
                     align-items: center; gap: 8px; user-select: none;
                     border-bottom: 1px solid var(--border); }}
  .section-header:hover {{ background: #f3f4f6; }}
  .section-header h2 {{ font-size: 1em; font-weight: 600; }}
  .section-header .badge {{ font-size: 0.8em; font-family: var(--mono); padding: 2px 8px;
                            border-radius: 10px; font-weight: 600; }}
  .section-header .arrow {{ color: var(--muted); font-size: 0.8em; transition: transform 0.2s; }}
  .section-body {{ padding: 0; }}
  .section.collapsed .section-body {{ display: none; }}
  .section.collapsed .arrow {{ transform: rotate(-90deg); }}
  .badge-pass {{ background: #dcfce7; color: var(--pass); }}
  .badge-fail {{ background: #fef2f2; color: var(--fail); }}
  .badge-warn {{ background: #fffbeb; color: var(--warn); }}
  .test-item {{ padding: 10px 16px; border-bottom: 1px solid var(--border);
                display: flex; align-items: flex-start; gap: 10px; font-size: 0.9em; }}
  .test-item:last-child {{ border-bottom: none; }}
  .icon {{ flex-shrink: 0; width: 20px; height: 20px; border-radius: 50%;
           display: flex; align-items: center; justify-content: center;
           font-size: 0.75em; font-weight: 700; color: #fff; }}
  .icon-pass {{ background: var(--pass); }}
  .icon-fail {{ background: var(--fail); }}
  .icon-warn {{ background: var(--warn); }}
  .test-name {{ font-family: var(--mono); font-weight: 500; word-break: break-all; }}
  .test-time {{ color: var(--muted); font-size: 0.85em; }}
  .diffs {{ margin-top: 8px; background: #1e1e2e; color: #cdd6f4; border-radius: 6px;
            padding: 10px 14px; font-family: var(--mono); font-size: 0.82em;
            line-height: 1.5; overflow-x: auto; }}
  .diff-add {{ color: #a6e3a1; }}
  .diff-remove {{ color: #f38ba8; }}
  .empty {{ padding: 16px; color: var(--muted); font-style: italic; }}
  footer {{ text-align: center; color: var(--muted); font-size: 0.8em; margin-top: 24px; }}
  footer a {{ color: var(--muted); }}
</style>
</head><body>
<div class="container">
{body}
<footer><a href="/">reg-rs</a> &middot; started {server_started}</footer>
</div>
<script>
document.querySelectorAll('.section').forEach(function(s) {{
  var badge = s.querySelector('.badge');
  if (badge && badge.textContent.trim() === '0') s.classList.add('collapsed');
}});
</script>
</body></html>"##
    )
}

/// Escape HTML special characters to prevent XSS
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Categorize test runs into failed, passed, and not-yet-run lists
fn categorize_runs(runs: &[TestDetails]) -> (Vec<String>, Vec<String>, Vec<String>) {
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

/// get test result differences
fn get_diffs(test_name: &str) -> crate::error::Result<Vec<String>> {
    log::info!("server/get_diffs test_name {}", &test_name);
    let differences = db::read_differences(test_name)?;
    let diffs = differences
        .iter()
        .filter_map(|(code, value)| format_difference(code, value))
        .collect();
    Ok(diffs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_difference_actual_code() {
        let result = format_difference("1", "42").unwrap();
        assert!(result.contains("diff-add"));
        assert!(result.contains("+ Actual code: 42"));
    }

    #[test]
    fn test_format_difference_expected_code() {
        let result = format_difference("2", "0").unwrap();
        assert!(result.contains("diff-remove"));
        assert!(result.contains("- Expected code: 0"));
    }

    #[test]
    fn test_format_difference_stderr() {
        let result = format_difference("3", "err").unwrap();
        assert!(result.contains("diff-add"));
        assert!(result.contains("+ Stderr add: err"));
        let result = format_difference("4", "old").unwrap();
        assert!(result.contains("diff-remove"));
        assert!(result.contains("- Stderr remove: old"));
    }

    #[test]
    fn test_format_difference_stdout() {
        let result = format_difference("6", "new").unwrap();
        assert!(result.contains("diff-add"));
        assert!(result.contains("+ Stdout add: new"));
        let result = format_difference("7", "old").unwrap();
        assert!(result.contains("diff-remove"));
        assert!(result.contains("- Stdout remove: old"));
    }

    #[test]
    fn test_format_difference_escapes_html() {
        let result = format_difference("6", "<script>alert('xss')</script>").unwrap();
        assert!(result.contains("&lt;script&gt;"));
        assert!(!result.contains("<script>alert"));
    }

    #[test]
    fn test_format_difference_skips_same() {
        assert_eq!(format_difference("5", "same"), None);
        assert_eq!(format_difference("8", "same"), None);
    }

    #[test]
    fn test_format_difference_unknown() {
        assert_eq!(format_difference("99", "data"), None);
        assert_eq!(format_difference("abc", "data"), None);
    }

    #[test]
    fn test_categorize_runs_empty() {
        let (failed, passed, not_yet_run) = categorize_runs(&[]);
        assert!(failed.is_empty());
        assert!(passed.is_empty());
        assert!(not_yet_run.is_empty());
    }

    #[test]
    fn test_categorize_runs_mixed() {
        let runs = vec![
            TestDetails {
                created: "2024-01-01".to_string(),
                diffs: None,
                last_ran: None,
                name: "not_run".to_string(),
            },
            TestDetails {
                created: "2024-01-01".to_string(),
                diffs: None,
                last_ran: Some("2024-01-02".to_string()),
                name: "passed".to_string(),
            },
            TestDetails {
                created: "2024-01-01".to_string(),
                diffs: Some(vec!["diff".to_string()]),
                last_ran: Some("2024-01-02".to_string()),
                name: "failed".to_string(),
            },
        ];
        let (failed, passed, not_yet_run) = categorize_runs(&runs);
        assert_eq!(failed, vec!["failed"]);
        assert_eq!(passed, vec!["passed"]);
        assert_eq!(not_yet_run, vec!["not_run"]);
    }
}

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{
        Html, IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::get,
};
use futures_util::stream::Stream;
use serde::Serialize;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

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
    /// Broadcast channel for SSE updates
    pub update_tx: broadcast::Sender<()>,
}

impl AppState {
    fn new(pattern: String) -> Self {
        let state_data = Arc::new(Mutex::new(StateData::new(pattern.clone())));
        let (update_tx, _) = broadcast::channel(16);
        Self {
            state_data,
            update_tx,
        }
    }

    /// Notify all SSE subscribers that state has changed
    pub fn notify_update(&self) {
        let _ = self.update_tx.send(());
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
        .route("/events", get(serve_sse))
        .with_state(app_state);

    log::info!("server/start - binding TCP listener to {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("server/start - TCP listener bound, starting axum::serve");

    axum::serve(listener, app.into_make_service()).await?;

    log::info!("server/start - END (server stopped)");
    Ok(())
}

/// Serve the landing page with summary and links to views
async fn serve_landing(State(state): State<AppState>) -> impl IntoResponse {
    // Refresh state before reading
    let _ = set_test_runs(state.clone());

    let (pattern, server_started, fail, pass, pending, total) = match state.state_data.lock() {
        Ok(guard) => {
            let (failed, passed, not_run) = categorize_runs(&guard.runs);
            (
                guard.pattern.clone(),
                guard.server_started.clone(),
                failed.len(),
                passed.len(),
                not_run.len(),
                guard.runs.len(),
            )
        }
        Err(_) => ("unknown".to_string(), String::new(), 0, 0, 0, 0),
    };

    let status_line = if fail > 0 {
        format!(
            r#"<span class="status-indicator fail">&#10007; {fail} failed</span>"#
        )
    } else if total == 0 {
        r#"<span class="status-indicator pending">No tests found</span>"#.to_string()
    } else if pending > 0 {
        format!(
            r#"<span class="status-indicator pending">? {pending} not yet run</span>"#
        )
    } else {
        format!(
            r#"<span class="status-indicator pass">&#10003; All {pass} tests passing</span>"#
        )
    };

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>reg-rs</title>
<style>
  :root {{
    --pass: #16a34a; --fail: #dc2626; --warn: #d97706; --muted: #6b7280;
    --bg: #f9fafb; --card: #fff; --border: #e5e7eb;
    --font: system-ui, -apple-system, sans-serif;
    --mono: ui-monospace, "SF Mono", Menlo, monospace;
  }}
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ font-family: var(--font); background: var(--bg); color: #111; padding: 40px 20px; }}
  .container {{ max-width: 600px; margin: 0 auto; }}
  h1 {{ font-size: 1.6em; margin-bottom: 4px; }}
  .meta {{ color: var(--muted); font-size: 0.85em; margin-bottom: 20px; }}
  .meta code {{ font-family: var(--mono); background: #f3f4f6; padding: 1px 5px;
               border-radius: 3px; }}
  .summary {{ background: var(--card); border: 1px solid var(--border); border-radius: 8px;
              padding: 20px; margin-bottom: 24px; }}
  .status-indicator {{ font-size: 1.1em; font-weight: 600; }}
  .status-indicator.pass {{ color: var(--pass); }}
  .status-indicator.fail {{ color: var(--fail); }}
  .status-indicator.pending {{ color: var(--warn); }}
  .counts {{ display: flex; gap: 20px; margin-top: 12px; font-size: 0.9em; color: var(--muted); }}
  .counts span {{ font-family: var(--mono); }}
  .views {{ list-style: none; padding: 0; }}
  .views li {{ margin-bottom: 12px; }}
  .views a {{ display: block; background: var(--card); border: 1px solid var(--border);
              border-radius: 8px; padding: 16px; text-decoration: none; color: #111;
              transition: border-color 0.15s; }}
  .views a:hover {{ border-color: #3b82f6; }}
  .views .view-title {{ font-size: 1.05em; font-weight: 600; }}
  .views .view-desc {{ color: var(--muted); font-size: 0.85em; margin-top: 4px; }}
  footer {{ text-align: center; color: var(--muted); font-size: 0.8em; margin-top: 32px; }}
</style>
</head><body>
<div class="container">
  <h1>reg-rs</h1>
  <div class="meta">pattern: <code>{pattern}</code></div>

  <div class="summary">
    {status_line}
    <div class="counts">
      <span>{pass} passed</span>
      <span>{fail} failed</span>
      <span>{pending} pending</span>
      <span>{total} total</span>
    </div>
  </div>

  <ul class="views">
    <li><a href="/status">
      <div class="view-title">Status Dashboard</div>
      <div class="view-desc">Pass/fail details, collapsible sections, diffs for failures</div>
    </a></li>
  </ul>

  <footer>started {server_started}</footer>
</div>
<script>
if (typeof EventSource !== 'undefined') {{
  var es = new EventSource('/events');
  es.onmessage = function() {{ location.reload(); }};
  es.onerror = function() {{ setTimeout(function() {{ es.close(); }}, 5000); }};
}}
</script>
</body></html>"##
    );
    (StatusCode::OK, Html(html))
}

/// Serve SSE stream for real-time updates
async fn serve_sse(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let mut rx = state.update_tx.subscribe();
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(()) => {
                    yield Ok(Event::default().data("update"));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    yield Ok(Event::default().data("update"));
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
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

/// Classify a difference tuple into a kind, label, and value.
/// Returns None for "same" types and unknown codes.
fn classify_difference(type_code: &str, value: &str) -> Option<(DiffKind, &'static str, String)> {
    let escaped = html_escape(value);
    match RegressionType::from_code(type_code)? {
        RegressionType::ActualCode => Some((DiffKind::Add, "Actual exit code", escaped)),
        RegressionType::ExpectedCode => Some((DiffKind::Remove, "Expected exit code", escaped)),
        RegressionType::StderrAdd => Some((DiffKind::Add, "Stderr", escaped)),
        RegressionType::StderrRemove => Some((DiffKind::Remove, "Stderr", escaped)),
        RegressionType::StdoutAdd => Some((DiffKind::Add, "Stdout", escaped)),
        RegressionType::StdoutRemove => Some((DiffKind::Remove, "Stdout", escaped)),
        RegressionType::StderrSame | RegressionType::StdoutSame => None,
    }
}

/// Diff direction
#[derive(Debug, PartialEq)]
enum DiffKind {
    /// Added (actual/new)
    Add,
    /// Removed (expected/baseline)
    Remove,
}

/// Format diffs into HTML with inline character highlighting.
/// Pairs up consecutive remove/add entries and highlights changed characters.
fn format_diffs_html(diffs: &[(String, String)]) -> Vec<String> {
    let classified: Vec<_> = diffs
        .iter()
        .filter_map(|(code, value)| classify_difference(code, value))
        .collect();

    let mut result = Vec::new();
    let mut i = 0;
    while i < classified.len() {
        let (kind, label, value) = &classified[i];
        // Try to pair a remove with the next add for inline highlighting
        if *kind == DiffKind::Remove && i + 1 < classified.len() && classified[i + 1].0 == DiffKind::Add {
            let (_, add_label, add_value) = &classified[i + 1];
            let (hl_old, hl_new) = highlight_diff(value, add_value);
            result.push(format!(
                r#"<div class="diff-pair"><div class="diff-label">{label}</div><div class="diff-remove">- expected: {hl_old}</div><div class="diff-add">+ actual:&nbsp;&nbsp; {hl_new}</div></div>"#
            ));
            let _ = add_label; // both labels shown via the pair
            i += 2;
        } else {
            let (class, prefix) = match kind {
                DiffKind::Add => ("diff-add", "+ actual"),
                DiffKind::Remove => ("diff-remove", "- expected"),
            };
            result.push(format!(
                r#"<div class="{class}">{prefix} ({label}): {value}</div>"#
            ));
            i += 1;
        }
    }
    result
}

/// Highlight character-level differences between two strings.
/// Returns (old_html, new_html) with `<mark>` around changed segments.
fn highlight_diff(old: &str, new: &str) -> (String, String) {
    // Find common prefix
    let prefix_len = old
        .chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .count();

    // Find common suffix (from the remaining chars after prefix)
    let old_rest: Vec<char> = old.chars().skip(prefix_len).collect();
    let new_rest: Vec<char> = new.chars().skip(prefix_len).collect();
    let suffix_len = old_rest
        .iter()
        .rev()
        .zip(new_rest.iter().rev())
        .take_while(|(a, b)| a == b)
        .count();

    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();

    let old_mid_end = old_chars.len().saturating_sub(suffix_len);
    let new_mid_end = new_chars.len().saturating_sub(suffix_len);

    let old_prefix: String = old_chars[..prefix_len].iter().collect();
    let old_mid: String = old_chars[prefix_len..old_mid_end].iter().collect();
    let old_suffix: String = old_chars[old_mid_end..].iter().collect();

    let new_prefix: String = new_chars[..prefix_len].iter().collect();
    let new_mid: String = new_chars[prefix_len..new_mid_end].iter().collect();
    let new_suffix: String = new_chars[new_mid_end..].iter().collect();

    if old_mid.is_empty() && new_mid.is_empty() {
        // Identical strings
        (old.to_string(), new.to_string())
    } else {
        (
            format!("{old_prefix}<mark>{old_mid}</mark>{old_suffix}"),
            format!("{new_prefix}<mark>{new_mid}</mark>{new_suffix}"),
        )
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
  .diff-pair {{ margin-bottom: 6px; }}
  .diff-label {{ color: #89b4fa; font-weight: 600; margin-bottom: 2px; }}
  .diffs mark {{ background: rgba(255,255,100,0.3); color: inherit; padding: 0 1px; border-radius: 2px; }}
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
if (typeof EventSource !== 'undefined') {{
  var es = new EventSource('/events');
  es.onmessage = function() {{ location.reload(); }};
  es.onerror = function() {{ setTimeout(function() {{ es.close(); }}, 5000); }};
}}
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
    Ok(format_diffs_html(&differences))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_difference_known_types() {
        let (kind, label, _) = classify_difference("1", "42").unwrap();
        assert_eq!(kind, DiffKind::Add);
        assert_eq!(label, "Actual exit code");

        let (kind, label, _) = classify_difference("2", "0").unwrap();
        assert_eq!(kind, DiffKind::Remove);
        assert_eq!(label, "Expected exit code");
    }

    #[test]
    fn test_classify_difference_skips_same() {
        assert!(classify_difference("5", "same").is_none());
        assert!(classify_difference("8", "same").is_none());
    }

    #[test]
    fn test_classify_difference_unknown() {
        assert!(classify_difference("99", "data").is_none());
        assert!(classify_difference("abc", "data").is_none());
    }

    #[test]
    fn test_classify_difference_escapes_html() {
        let (_, _, value) = classify_difference("6", "<b>xss</b>").unwrap();
        assert!(value.contains("&lt;b&gt;"));
        assert!(!value.contains("<b>xss"));
    }

    #[test]
    fn test_format_diffs_html_pairs_remove_add() {
        let diffs = vec![
            ("7".to_string(), "old_value".to_string()),
            ("6".to_string(), "new_value".to_string()),
        ];
        let result = format_diffs_html(&diffs);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("diff-pair"));
        assert!(result[0].contains("expected"));
        assert!(result[0].contains("actual"));
    }

    #[test]
    fn test_highlight_diff_marks_changed_chars() {
        let (old_html, new_html) = highlight_diff("abc123def", "abc456def");
        assert!(old_html.contains("<mark>123</mark>"));
        assert!(new_html.contains("<mark>456</mark>"));
    }

    #[test]
    fn test_highlight_diff_identical() {
        let (old_html, new_html) = highlight_diff("same", "same");
        assert!(!old_html.contains("<mark>"));
        assert!(!new_html.contains("<mark>"));
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

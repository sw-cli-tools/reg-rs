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
        .route("/", get(serve_status_view))
        .with_state(app_state);

    log::info!("server/start - binding TCP listener to {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("server/start - TCP listener bound, starting axum::serve");

    axum::serve(listener, app.into_make_service()).await?;

    log::info!("server/start - END (server stopped)");
    Ok(())
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

    let status_view = match status::render(&status::StatusViewContext::new(
        state_data.server_started.clone(),
        state_data.state_updated.clone(),
        status_counts,
        status_flags,
        state_data.pattern.to_string(),
        &state_data.runs,
    )) {
        Ok(view) => view,
        Err(e) => {
            log::error!("Failed to render status view: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Error</h1>".to_string()),
            );
        }
    };
    let response_str = format!("<div>{}</div>", status_view);
    log::info!(
        "server/serve_status_view - END, response len={}",
        response_str.len()
    );
    (StatusCode::OK, Html(response_str))
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
        if latest_results_row_count == 0 {
            test_runs.push(TestDetails {
                created: original_result.time_created,
                diffs: None,
                name: test_name.to_string(),
                last_ran: None,
            });
        } else {
            let latest_result = db::read_latest_results(test_name)?;
            let difference_count = db::count_differences(test_name)?;
            if difference_count > 0 {
                test_runs.push(TestDetails {
                    created: original_result.time_created,
                    diffs: Some(get_diffs(test_name)?),
                    name: test_name.to_string(),
                    last_ran: Some(latest_result.time_created),
                });
            } else {
                test_runs.push(TestDetails {
                    created: original_result.time_created,
                    diffs: None,
                    name: test_name.to_string(),
                    last_ran: Some(latest_result.time_created),
                });
            }
        }
    }
    state_data.runs = test_runs;
    state_data.state_updated = time::now();
    log::info!(
        "server/set_test_runs - completed, {} tests loaded, releasing lock",
        state_data.runs.len()
    );
    Ok(())
}

/// Format a single difference tuple into a display string.
/// Returns None for "same" types (5, 8) that should be skipped.
fn format_difference(type_code: &str, value: &str) -> Option<String> {
    match type_code {
        "5" | "8" => None,
        "1" => Some(format!("+ Actual code: {}", value)),
        "2" => Some(format!("- Expected code: {}", value)),
        "3" => Some(format!("+ Stderr add: {}", value)),
        "4" => Some(format!("- Stderr remove: {}", value)),
        "6" => Some(format!("+ Stdout add: {}", value)),
        "7" => Some(format!("- Stdout remove: {}", value)),
        _ => Some(format!("not yet implemented: {} {}", type_code, value)),
    }
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
        assert_eq!(
            format_difference("1", "42"),
            Some("+ Actual code: 42".to_string())
        );
    }

    #[test]
    fn test_format_difference_expected_code() {
        assert_eq!(
            format_difference("2", "0"),
            Some("- Expected code: 0".to_string())
        );
    }

    #[test]
    fn test_format_difference_stderr() {
        assert_eq!(
            format_difference("3", "err"),
            Some("+ Stderr add: err".to_string())
        );
        assert_eq!(
            format_difference("4", "old"),
            Some("- Stderr remove: old".to_string())
        );
    }

    #[test]
    fn test_format_difference_stdout() {
        assert_eq!(
            format_difference("6", "new"),
            Some("+ Stdout add: new".to_string())
        );
        assert_eq!(
            format_difference("7", "old"),
            Some("- Stdout remove: old".to_string())
        );
    }

    #[test]
    fn test_format_difference_skips_same() {
        assert_eq!(format_difference("5", "same"), None);
        assert_eq!(format_difference("8", "same"), None);
    }

    #[test]
    fn test_format_difference_unknown() {
        assert_eq!(
            format_difference("99", "data"),
            Some("not yet implemented: 99 data".to_string())
        );
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

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::config;
use crate::db;
use crate::error::RttError;
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
        let data_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("data");
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
pub async fn start(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
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

    let mut failed_test_names: Vec<String> = vec![];
    let mut not_yet_run_test_names: Vec<String> = vec![];
    let mut passed_test_names: Vec<String> = vec![];

    for run in &state_data.runs {
        if run.last_ran.is_none() {
            not_yet_run_test_names.push(run.name.clone());
        } else if run.diffs.is_none() {
            passed_test_names.push(run.name.clone());
        } else {
            failed_test_names.push(run.name.clone());
        }
    }

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
pub fn set_test_runs(app_state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("server/set_test_runs - acquiring lock");
    let mut state_data = app_state
        .state_data
        .lock()
        .map_err(|e| RttError::MutexPoisoned(format!("state_data lock failed: {}", e)))?;
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

/// get test result differences
fn get_diffs(test_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    log::info!("server/get_diffs test_name {}", &test_name);
    let differences = db::read_differences(test_name)?;
    let mut diffs = vec![];
    for difference in differences {
        match difference.0.as_ref() {
            "5" => continue,
            "8" => continue,
            _ => (),
        }
        diffs.push(match difference.0.as_ref() {
            "1" => format!("+ Actual code: {}", difference.1).to_string(),
            "2" => format!("- Expected code: {}", difference.1).to_string(),
            "3" => format!("+ Stderr add: {}", difference.1).to_string(),
            "4" => format!("- Stderr remove: {}", difference.1).to_string(),
            //   "5" => format!("= Stderr same: {}", difference.1).to_string(),
            "6" => format!("+ Stdout add: {}", difference.1).to_string(),
            "7" => format!("- Stdout remove: {}", difference.1).to_string(),
            //   "8" => format!("= Stdout same: {}", difference.1).to_string(),
            _ => format!("not yet implemented: {} {}", difference.0, difference.1).to_string(),
        });
    }
    Ok(diffs)
}

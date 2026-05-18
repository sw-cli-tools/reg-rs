use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::Serialize;

use reg_rs_renderer::templates;

use crate::state::AppState;

#[derive(Serialize)]
/// API status response payload
pub struct ApiStatusResponse {
    pattern: String,
    tests: Vec<templates::TestDetails>,
    fail_count: usize,
    pass_count: usize,
    pending_count: usize,
    total_count: usize,
    updated: String,
}

/// Serve the landing page
pub async fn serve_landing(State(state): State<AppState>) -> impl IntoResponse {
    refresh_state(&state);

    let (pattern, server_started, fail, pass, pending, total, data_dir) =
        match state.state_data.lock() {
            Ok(guard) => {
                let (failed, passed, not_run) = templates::categorize_runs(&guard.runs);
                (
                    guard.pattern.clone(),
                    guard.server_started.clone(),
                    failed.len(),
                    passed.len(),
                    not_run.len(),
                    guard.runs.len(),
                    guard.data_dir.display().to_string(),
                )
            }
            Err(_) => (
                "unknown".to_string(),
                String::new(),
                0,
                0,
                0,
                0,
                String::new(),
            ),
        };

    let html = templates::landing_page(
        &pattern,
        &server_started,
        fail,
        pass,
        pending,
        total,
        &data_dir,
    );
    (StatusCode::OK, Html(html))
}

/// Serve the JSON API status endpoint
pub async fn serve_api_status(State(state): State<AppState>) -> impl IntoResponse {
    refresh_state(&state);
    match state.state_data.lock() {
        Ok(guard) => {
            let (failed, passed, not_run) = templates::categorize_runs(&guard.runs);
            let resp = ApiStatusResponse {
                pattern: guard.pattern.clone(),
                tests: guard.runs.clone(),
                fail_count: failed.len(),
                pass_count: passed.len(),
                pending_count: not_run.len(),
                total_count: guard.runs.len(),
                updated: guard.state_updated.clone(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Serve SSE event stream with JSON data payload
pub async fn serve_sse(
    State(state): State<AppState>,
) -> axum::response::sse::Sse<
    impl futures_util::stream::Stream<
        Item = Result<axum::response::sse::Event, std::convert::Infallible>,
    >,
> {
    let mut rx = state.update_tx.subscribe();
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(changed) => {
                    refresh_state(&state);
                    let json = build_sse_json(&state, &changed);
                    yield Ok(axum::response::sse::Event::default().data(json));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    refresh_state(&state);
                    let json = build_sse_json(&state, "");
                    yield Ok(axum::response::sse::Event::default().data(json));
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    axum::response::sse::Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

/// Build JSON payload for SSE events
fn build_sse_json(state: &AppState, changed_test: &str) -> String {
    match state.state_data.lock() {
        Ok(guard) => {
            let (failed, passed, not_run) = templates::categorize_runs(&guard.runs);
            let test = changed_test.replace('"', "");
            format!(
                r#"{{"pass":{},"fail":{},"pending":{},"total":{},"test":"{}"}}"#,
                passed.len(),
                failed.len(),
                not_run.len(),
                guard.runs.len(),
                test
            )
        }
        Err(_) => r#"{"pass":0,"fail":0,"pending":0,"total":0,"test":""}"#.to_string(),
    }
}

/// Serve the status dashboard view
pub async fn serve_status_view(State(state): State<AppState>) -> impl IntoResponse {
    log::info!("server/serve_status_view - START");
    refresh_state(&state);

    let state_data = match state.state_data.lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock state data: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Error: mutex poisoned</h1>".to_string()),
            );
        }
    };

    match build_status_page(&state_data) {
        Ok(page) => (StatusCode::OK, Html(page)),
        Err(e) => {
            log::error!("Failed to render status view: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Error</h1>".to_string()),
            )
        }
    }
}

/// Build the full status dashboard HTML from current state data.
fn build_status_page(state_data: &crate::state::StateData) -> reg_rs_types::error::Result<String> {
    let (failed, passed, not_run) = templates::categorize_runs(&state_data.runs);
    let status_counts = templates::StatusCounts {
        fail_count: format!(" {:05}", failed.len()),
        not_run_count: format!(" {:05}", not_run.len()),
        pass_count: format!(" {:05}", passed.len()),
        test_count: format!(" {:05}", state_data.runs.len()),
    };
    let status_flags = templates::StatusFlags {
        no_failed_tests: failed.is_empty(),
        no_not_yet_run_tests: not_run.is_empty(),
        no_passed_tests: passed.is_empty(),
    };
    let ctx = templates::StatusViewContext::new(
        state_data.server_started.clone(),
        state_data.state_updated.clone(),
        status_counts,
        status_flags,
        state_data.pattern.to_string(),
        &state_data.runs,
    );
    let status_view = templates::render_status_view(&ctx)?;
    Ok(templates::status_page(
        &state_data.pattern,
        &state_data.server_started,
        &status_view,
    ))
}

/// Refresh state by collecting test runs from database.
///
/// Uses the stable test_paths list (set once at startup) to re-read
/// current results from disk on every request.
fn refresh_state(state: &AppState) {
    let test_paths: Vec<String> = match state.state_data.lock() {
        Ok(guard) => guard.test_paths.clone(),
        Err(_) => return,
    };

    if test_paths.is_empty() {
        return;
    }

    match reg_rs_renderer::test_runner::collect_test_runs(&test_paths) {
        Ok(mut runs) => {
            // Sort: most recently run first, pending (no last_ran) at bottom
            runs.sort_by(|a, b| match (&b.last_ran, &a.last_ran) {
                (Some(b_ts), Some(a_ts)) => b_ts.cmp(a_ts),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.cmp(&b.name),
            });
            if let Ok(mut guard) = state.state_data.lock() {
                guard.runs = runs;
                let date = chrono::Local::now();
                guard.state_updated = format!("{}", date.format("%Y-%m-%dT%H:%M:%S%:z"));
            }
        }
        Err(e) => log::error!("Failed to refresh test runs: {e}"),
    }
}

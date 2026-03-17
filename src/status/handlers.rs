use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::Serialize;

use super::state::AppState;
use super::status_builder;
use super::templates::{categorize_runs, landing_page, status_page};
use super::test_runner::set_test_runs;

#[derive(Serialize)]
pub struct ApiStatusResponse {
    pattern: String,
    tests: Vec<super::state::TestDetails>,
    fail_count: usize,
    pass_count: usize,
    pending_count: usize,
    total_count: usize,
    updated: String,
}

pub async fn serve_landing(State(state): State<AppState>) -> impl IntoResponse {
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

    let html = landing_page(&pattern, &server_started, fail, pass, pending, total);
    (StatusCode::OK, Html(html))
}

pub async fn serve_api_status(State(state): State<AppState>) -> impl IntoResponse {
    let _ = set_test_runs(state.clone());
    match state.state_data.lock() {
        Ok(guard) => {
            let (failed, passed, not_run) = categorize_runs(&guard.runs);
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
                Ok(()) => {
                    yield Ok(axum::response::sse::Event::default().data("update"));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    yield Ok(axum::response::sse::Event::default().data("update"));
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    axum::response::sse::Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

pub async fn serve_status_view(State(state): State<AppState>) -> impl IntoResponse {
    log::info!("server/serve_status_view - START");

    if let Err(e) = set_test_runs(state.clone()) {
        log::error!("Failed to update test runs: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<h1>Error updating test runs</h1>".to_string()),
        );
    }

    let state_data = match state.state_data.lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock state data: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Error: mutex poisoned</h1>".to_string()),
            );
        }
    };

    let status_view = match status_builder::build_status_view(&state_data) {
        Ok(view) => view,
        Err(e) => {
            log::error!("Failed to render status view: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Error</h1>".to_string()),
            );
        }
    };
    let page = status_page(
        &state_data.pattern,
        &state_data.server_started,
        &status_view,
    );
    (StatusCode::OK, Html(page))
}

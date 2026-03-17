use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{Router, routing::get};

use crate::handlers;
use crate::state::AppState;

/// Start HTTP status server.
///
/// `initial_tests` provides the list of discovered test paths to pre-populate state.
pub async fn start(
    pattern: &str,
    port: u16,
    data_dir: PathBuf,
    initial_tests: &[String],
) -> reg_rs_types::error::Result<()> {
    log::info!("server/start - BEGIN");
    log::info!("server/start - port={}, pattern={}", port, pattern);

    let app_state = AppState::new(pattern.to_string(), data_dir);

    // Pre-populate state with discovered tests
    if !initial_tests.is_empty()
        && let Ok(runs) = reg_rs_renderer::test_runner::collect_test_runs(initial_tests)
        && let Ok(mut guard) = app_state.state_data.lock()
    {
        guard.runs = runs;
    }

    let _handles = crate::monitor::launch_monitor(app_state.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Listening at {}.  Ctrl-C to terminate server", addr);

    let app = Router::new()
        .route("/", get(handlers::serve_landing))
        .route("/status", get(handlers::serve_status_view))
        .route("/api/status", get(handlers::serve_api_status))
        .route("/events", get(handlers::serve_sse))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| reg_rs_types::error::RegError::WebServer(e.to_string()))?;

    log::info!("server/start - END (server stopped)");
    Ok(())
}

use std::net::SocketAddr;

use axum::{Router, routing::get};

use crate::config;
use crate::status::{AppState, test_runner};

use super::handlers;

/// Start HTTP status server
pub async fn start(config: &config::Config) -> crate::error::Result<()> {
    log::info!("server/start - BEGIN");
    let status_port = config.status_port();
    let pattern = config.extract_pattern().to_string();
    log::info!("server/start - port={}, pattern={}", status_port, pattern);

    let app_state = AppState::new(pattern.clone());

    test_runner::set_test_runs(app_state.clone())?;

    let _handles = super::monitor::launch_monitor(app_state.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], status_port));
    println!("Listening at {}.  Ctrl-C to terminate server", addr);

    let app = Router::new()
        .route("/", get(handlers::serve_landing))
        .route("/status", get(handlers::serve_status_view))
        .route("/api/status", get(handlers::serve_api_status))
        .route("/events", get(handlers::serve_sse))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service()).await?;

    log::info!("server/start - END (server stopped)");
    Ok(())
}

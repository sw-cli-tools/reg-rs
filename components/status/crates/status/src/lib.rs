//! Axum web server for reg-rs status monitoring.
#![deny(warnings, missing_docs)]

/// Request handlers for landing, status, API, and SSE endpoints
pub mod handlers;
/// File system monitoring for test result changes
pub mod monitor;
/// HTTP server setup and routing
pub mod server;
/// Shared application state (AppState, StateData)
pub mod state;

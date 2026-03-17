/// Status monitoring module for web UI
pub mod client;
/// Diff formatting for web UI
pub mod diff_formatter;
mod event_loop;
mod handlers;
/// File monitoring for status updates
pub mod monitor;
/// Web server for status dashboard
pub mod server;
mod state;
/// Status view HTML builder
pub mod status_builder;
mod templates;
mod test_runner;
mod watcher_setup;

pub use state::{AppState, StateData, TestDetails};
pub use templates::{StatusCounts, StatusFlags, StatusViewContext, render};
pub use test_runner::set_test_runs;

pub use client::start as start_client;
pub use server::start as start_server;

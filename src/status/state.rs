use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use crate::time;

/// Shared application state for the status server
#[derive(Clone)]
pub struct AppState {
    /// Shared state data
    pub state_data: Arc<Mutex<StateData>>,
    /// Broadcast channel for SSE updates
    pub update_tx: broadcast::Sender<()>,
}

impl AppState {
    /// Create new app state
    pub fn new(pattern: String) -> Self {
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

/// Data stored in the shared application state
#[derive(Debug)]
pub struct StateData {
    /// Test name pattern
    pub pattern: String,
    /// List of test results
    pub runs: Vec<TestDetails>,
    /// Server start time
    pub server_started: String,
    /// Last state update time
    pub state_updated: String,
    /// Data directory to watch
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

/// Details about a single test run
#[derive(Debug, serde::Serialize, Clone)]
pub struct TestDetails {
    /// When test was first run
    pub created: String,
    /// Test differences from last run (if any)
    pub diffs: Option<Vec<String>>,
    /// When test was last run (if more than once)
    pub last_ran: Option<String>,
    /// Test name
    pub name: String,
}

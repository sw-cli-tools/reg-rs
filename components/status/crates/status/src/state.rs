use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

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
    pub fn new(pattern: String, data_dir: PathBuf) -> Self {
        let state_data = Arc::new(Mutex::new(StateData::new(pattern, data_dir)));
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
    /// Discovered test file paths (stable, set once at startup)
    pub test_paths: Vec<String>,
    /// List of test results (refreshed on each request)
    pub runs: Vec<reg_rs_renderer::templates::TestDetails>,
    /// Server start time
    pub server_started: String,
    /// Last state update time
    pub state_updated: String,
    /// Data directory to watch
    pub data_dir: PathBuf,
}

impl StateData {
    fn new(pattern: String, data_dir: PathBuf) -> Self {
        let date = chrono::Local::now();
        let now = format!("{}", date.format("%Y-%m-%dT%H:%M:%S"));
        Self {
            pattern,
            test_paths: vec![],
            runs: vec![],
            server_started: now,
            state_updated: "".to_string(),
            data_dir,
        }
    }
}

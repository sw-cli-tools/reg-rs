use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use notify::{RecursiveMode, Watcher, watcher};

use crate::error::RegError;
use crate::status::server::{self, AppState};
use crate::time;

/// Launch monitor thread.
///
/// The monitor watches the data directory for file changes and updates
/// the shared application state. Errors during setup are logged and
/// the thread exits gracefully rather than panicking.
pub fn launch_monitor(app_state: AppState) -> Vec<std::thread::JoinHandle<()>> {
    log::info!("monitor/launch_monitor");
    let mut handles = vec![];
    handles.push({
        thread::spawn(move || {
            if let Err(e) = watch(app_state) {
                log::error!("monitor/watch exited with error: {}", e);
                eprintln!("Warning: file monitor stopped: {}", e);
            }
        })
    });
    handles
}

/// Watch for test results, returning an error instead of panicking on failure
fn watch(app_state: AppState) -> Result<(), RegError> {
    let (pattern, data_dir) = {
        let state_data = app_state
            .state_data
            .lock()
            .map_err(|e| RegError::MutexPoisoned(format!("monitor startup: {}", e)))?;
        (state_data.pattern.clone(), state_data.data_dir.clone())
    };
    log::info!("monitor/watch pattern: {}", &pattern);
    log::info!("monitor/watch data_dir: {:?}", &data_dir);
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(5))
        .map_err(|e| RegError::Notification(format!("failed to create watcher: {}", e)))?;
    watcher
        .watch(&data_dir, RecursiveMode::Recursive)
        .map_err(|e| {
            RegError::Notification(format!("failed to watch directory {:?}: {}", data_dir, e))
        })?;

    let mut index = 0;
    loop {
        index += 1;
        match rx.recv() {
            Ok(event) => log::debug!("monitor/watch event: {:?}", event),
            Err(e) => {
                log::error!("monitor/watch: channel closed: {}", e);
                return Err(RegError::Notification(format!(
                    "file watcher channel closed: {}",
                    e
                )));
            }
        }
        {
            match app_state.state_data.lock() {
                Ok(mut state_data) => {
                    state_data.state_updated = time::now();
                    log::info!(
                        "monitor/watch state_data.state_updated: {}",
                        &state_data.state_updated
                    );
                    log::debug!("monitor state_updated: {}", &state_data.state_updated);
                }
                Err(e) => {
                    log::error!("monitor/watch: Failed to lock state_data in loop: {}", e);
                }
            }
        }
        if let Err(e) = server::set_test_runs(app_state.clone()) {
            log::error!("monitor/watch: Failed to set test runs: {}", e);
        }
        log::debug!("monitor loop {}", index);
    }
}

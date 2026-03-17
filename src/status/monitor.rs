use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::RegError;
use crate::status::{AppState, test_runner};

use super::event_loop;
use super::watcher_setup;

/// Launch monitor thread.
pub(crate) fn launch_monitor(app_state: AppState) -> Vec<std::thread::JoinHandle<()>> {
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

/// Watch for test results
fn watch(app_state: AppState) -> Result<(), RegError> {
    let (_, data_dir) = watcher_setup::extract_watch_info(&app_state)?;
    log::info!("monitor/watch data_dir: {:?}", &data_dir);

    let (tx, rx) = channel();
    let mut watcher = notify::watcher(tx, Duration::from_secs(2))
        .map_err(|e| RegError::Notification(format!("failed to create watcher: {}", e)))?;

    watcher
        .watch(&data_dir, RecursiveMode::Recursive)
        .map_err(|e| {
            RegError::Notification(format!("failed to watch directory {:?}: {}", data_dir, e))
        })?;

    run_event_loop(&app_state, &mut watcher, rx)
}

/// Run the main event loop for file system monitoring
fn run_event_loop(
    app_state: &AppState,
    _watcher: &mut RecommendedWatcher,
    rx: std::sync::mpsc::Receiver<notify::DebouncedEvent>,
) -> Result<(), RegError> {
    let mut index = event_loop::init_loop_index();
    loop {
        index = event_loop::increment_and_continue(index);

        match rx.recv() {
            Ok(event) => {
                event_loop::process_event(event);

                if update_state_and_timestamp(app_state) {
                    if let Err(e) = test_runner::set_test_runs(app_state.clone()) {
                        log::error!("monitor/watch: Failed to set test runs: {}", e);
                    }
                    app_state.notify_update();
                }
            }
            Err(e) => {
                log::error!("monitor/watch: channel closed: {}", e);
                return Err(RegError::Notification(format!(
                    "file watcher channel closed: {}",
                    e
                )));
            }
        }

        log::debug!("monitor loop {}", index);
    }
}

/// Update state data with current timestamp
fn update_state_and_timestamp(app_state: &AppState) -> bool {
    match app_state.state_data.lock() {
        Ok(mut state_data) => {
            event_loop::update_timestamp(&mut state_data);
            true
        }
        Err(e) => {
            log::error!("monitor/watch: Failed to lock state_data in loop: {}", e);
            false
        }
    }
}

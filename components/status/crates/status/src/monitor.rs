use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};

use reg_rs_types::error::RegError;

use notify::DebouncedEvent;

use crate::state::AppState;

/// Extract the file path from a notify event, returning the stem as the test name.
fn extract_event_path(event: &DebouncedEvent) -> String {
    let path = match event {
        DebouncedEvent::Write(p)
        | DebouncedEvent::Create(p)
        | DebouncedEvent::Remove(p)
        | DebouncedEvent::Rename(_, p)
        | DebouncedEvent::NoticeWrite(p)
        | DebouncedEvent::NoticeRemove(p)
        | DebouncedEvent::Chmod(p) => Some(p),
        _ => None,
    };
    path.and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

/// Launch monitor thread.
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

/// Watch for test result changes
fn watch(app_state: AppState) -> Result<(), RegError> {
    let raw_dir = match app_state.state_data.lock() {
        Ok(guard) => guard.data_dir.clone(),
        Err(e) => {
            return Err(RegError::MutexPoisoned(format!("monitor startup: {}", e)));
        }
    };
    // Canonicalize to absolute path — notify watcher needs this for reliable event delivery
    let data_dir = raw_dir.canonicalize().unwrap_or_else(|_| raw_dir.clone());
    log::info!("monitor/watch data_dir: {:?}", &data_dir);

    let (tx, rx) = channel();
    let mut watcher = notify::watcher(tx, Duration::from_secs(2))
        .map_err(|e| RegError::Notification(format!("failed to create watcher: {}", e)))?;

    watcher
        .watch(&data_dir, RecursiveMode::Recursive)
        .map_err(|e| {
            RegError::Notification(format!("failed to watch directory {:?}: {}", data_dir, e))
        })?;

    run_event_loop(&app_state, rx)
}

/// Run the main event loop for file system monitoring
fn run_event_loop(
    app_state: &AppState,
    rx: std::sync::mpsc::Receiver<notify::DebouncedEvent>,
) -> Result<(), RegError> {
    loop {
        match rx.recv() {
            Ok(ref event) => {
                log::debug!("monitor/watch event: {:?}", event);
                let path = extract_event_path(event);
                app_state.notify_update(path);
            }
            Err(e) => {
                log::error!("monitor/watch: channel closed: {}", e);
                return Err(RegError::Notification(format!(
                    "file watcher channel closed: {}",
                    e
                )));
            }
        }
    }
}

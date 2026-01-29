use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use notify::{RecursiveMode, Watcher, watcher};

use crate::status::server::{self, AppState};
use crate::time;

/// launch monitor thread
pub fn launch_monitor(app_state: AppState) -> Vec<std::thread::JoinHandle<()>> {
    log::info!("monitor/launch_monitor");
    let mut handles = vec![];
    handles.push({
        thread::spawn(move || {
            watch(app_state);
        })
    });
    handles
}

/// watch for test results
fn watch(app_state: AppState) -> ! {
    let (pattern, data_dir) = match app_state.state_data.lock() {
        Ok(state_data) => (state_data.pattern.clone(), state_data.data_dir.clone()),
        Err(e) => {
            log::error!("monitor/watch: Failed to lock state_data: {}", e);
            panic!("Cannot start monitor: mutex poisoned");
        }
    };
    log::info!("monitor/watch pattern: {}", &pattern);
    log::info!("monitor/watch data_dir: {:?}", &data_dir);
    let (tx, rx) = channel();
    let mut watcher = match watcher(tx, Duration::from_secs(5)) {
        Ok(w) => w,
        Err(e) => {
            log::error!("monitor/watch: Failed to create watcher: {}", e);
            panic!("Cannot start monitor: watcher creation failed");
        }
    };
    if let Err(e) = watcher.watch(&data_dir, RecursiveMode::Recursive) {
        log::error!(
            "monitor/watch: Failed to watch directory {:?}: {}",
            data_dir,
            e
        );
        panic!("Cannot start monitor: directory watch failed");
    }

    let mut index = 0;
    loop {
        index += 1;
        match rx.recv() {
            Ok(event) => println!("monitor.rs {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
        {
            match app_state.state_data.lock() {
                Ok(mut state_data) => {
                    state_data.state_updated = time::now();
                    log::info!(
                        "monitor/watch state_data.state_updated: {}",
                        &state_data.state_updated
                    );
                    md!(&state_data.state_updated);
                }
                Err(e) => {
                    log::error!("monitor/watch: Failed to lock state_data in loop: {}", e);
                }
            }
        }
        if let Err(e) = server::set_test_runs(app_state.clone()) {
            log::error!("monitor/watch: Failed to set test runs: {}", e);
        }
        md!(format!("loop {}", index));
    }
}

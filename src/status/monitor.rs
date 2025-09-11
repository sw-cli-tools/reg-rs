use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use notify::{watcher, RecursiveMode, Watcher};

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
    let pattern = app_state.state_data.lock().unwrap().pattern.clone();
    log::info!("monitor/watch pattern: {}", &pattern);
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(5)).unwrap();
    // TODO use cwd
    watcher
        .watch(
            "/home/mike/github/wrightmikea/rtt1/data", // TODO fixme
            RecursiveMode::Recursive,
        )
        .unwrap();

    let mut index = 0;
    loop {
        index += 1;
        match rx.recv() {
            Ok(event) => println!("monitor.rs {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
        {
            let mut state_data = app_state.state_data.lock().unwrap();
            state_data.state_updated = time::now();
            log::info!(
                "monitor/watch state_data.state_updated: {}",
                &state_data.state_updated
            );

            md!(&state_data.state_updated);
        }
        server::set_test_runs(app_state.clone()).unwrap();
        md!(format!("loop {}", index));
    }
}
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use notify::{watcher, RecursiveMode, Watcher};

use crate::status::server;
use crate::time;

pub fn launch_monitor(pattern: String) -> Vec<std::thread::JoinHandle<()>> {
    log::info!("monitor/launch_monitor pattern: {}", &pattern);
    let state_data = &server::STATE_DATA.lock().unwrap();
    md!(&state_data);
    let mut handles = vec![];
    handles.push({
        thread::spawn(move || {
            watch(&pattern);
        })
    });
    handles
}

fn watch(pattern: &str) -> ! {
    log::info!("monitor/watch pattern: {}", &pattern);
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(5)).unwrap();
    // TODO use cwd
    watcher
        .watch(
            "/home/mike/github/wrightmikea/rtt1/data",
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
            let mut state_data = server::STATE_DATA.lock().unwrap();
            state_data.state_updated = time::now();
            log::info!(
                "monitor/watch state_data.state_updated: {}",
                &state_data.state_updated
            );

            md!(&state_data.state_updated);
        }
        server::set_test_runs(pattern.to_string()).unwrap();
        md!(format!("loop {}", index));
    }
}

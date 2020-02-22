use std::thread;
use std::time::{Duration, SystemTime};

use crate::status::server;
use crate::time;

pub fn launch_monitor(pattern: String) -> Vec<std::thread::JoinHandle<()>> {
    let state_data = &server::STATE_DATA.lock().unwrap();
    md!(&state_data);
    let mut handles = vec![];
    handles.push({
        let now = SystemTime::now();
        thread::spawn(move || {
            for index in 1..=15 {
                thread::sleep(Duration::from_secs(5));
                println!("awake #{}, elapsed: {}", index, now.elapsed().unwrap().as_secs());
                {
                    let mut state_data = server::STATE_DATA.lock().unwrap();
                    state_data.state_updated = time::now();
                    md!(&state_data.state_updated);
                }
                server::set_test_runs(pattern.to_string()).unwrap();
            }
        })
    });
    handles
}

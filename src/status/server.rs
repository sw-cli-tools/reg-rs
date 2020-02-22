use std::sync::Mutex;

use gotham::state::State;
use mime;

use crate::config;
use crate::db;
use crate::finder;
use crate::status::monitor;
use crate::status::views::status;
use crate::time;

lazy_static! {
    pub static ref STATE_DATA: Mutex<StateData> = Mutex::new(StateData {
        pattern: "".to_string(),
        runs: vec![],
        server_started: time::now(),
        state_updated: "".to_string(),
    });
}

#[derive(Debug)]
pub struct StateData {
    pub pattern: String,
    pub runs: Vec<TestDetails>,
    server_started: String,
    pub state_updated: String,
}

#[derive(Debug, Serialize)]
pub struct TestDetails {
    pub created: String,
    pub diffs: Option<Vec<String>>,
    pub last_ran: Option<String>,
    pub name: String,
}

pub fn start(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let status_port = config.status_port();
    let pattern = config.extract_pattern().to_string();
    set_test_runs(pattern.to_string())?;
    let handles = monitor::launch_monitor(pattern.to_string());
    let addr = format!("localhost:{}", status_port);
    println!("Listening at {}.  Ctrl-C to terminate server", addr);
    gotham::start(addr, || Ok(serve_status_view)); // loops until Ctrl-C kills process
    for handle in handles {
        handle.join().unwrap();
    }
    Ok(())
}

fn serve_status_view(state: State) -> (State, (mime::Mime, String)) {
    let state_data = &STATE_DATA.lock().unwrap();
    md!(&state_data.state_updated);
    let mut failed_test_names = vec![];
    let mut not_yet_run_test_names = vec![];
    let mut passed_test_names = vec![];

    let mut copied_runs = vec![];
    for run in &state_data.runs {
        copied_runs.push(TestDetails {
            created: run.created.clone(),
            diffs: run.diffs.clone(),
            last_ran: run.last_ran.clone(),
            name: run.name.clone(),
        });
        if run.last_ran.is_none() {
            not_yet_run_test_names.push(run.name.clone());
        } else if run.diffs.is_none() {
            passed_test_names.push(run.name.clone());
        } else {
            failed_test_names.push(run.name.clone());
        }
    }
    let status_view = status::render(&status::StatusViewContext::new(
        failed_test_names.len() as u32,
        failed_test_names.len() == 0 as usize,
        not_yet_run_test_names.len() == 0 as usize,
        passed_test_names.len() == 0 as usize,
        not_yet_run_test_names.len() as u32,
        passed_test_names.len() as u32,
        state_data.server_started.clone(),
        state_data.state_updated.clone(),
        state_data.runs.len() as u32,
        state_data.pattern.to_string(),
        copied_runs,
    ))
    .unwrap();
    let response_str = format!("<div>{}</div>", status_view);
    (state, (mime::TEXT_HTML, response_str.to_string()))
}

pub fn set_test_runs(pattern: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut test_runs = vec![];
    let test_names = finder::discover(pattern.to_string())?;
    for test_name in &test_names.found {
        let original_result = db::read_original_results(&test_name)?;
        let latest_results_table_count = db::latest_results_table_count(&test_name)?;
        if latest_results_table_count == 0 {
            test_runs.push(TestDetails {
                created: original_result.time_created,
                diffs: None,
                name: test_name.to_string(),
                last_ran: None,
            });
        } else {
            let latest_result = db::read_latest_results(&test_name)?;
            let difference_count = db::count_differences(&test_name)?;
            if difference_count > 0 {
                test_runs.push(TestDetails {
                    created: original_result.time_created,
                    diffs: Some(get_diffs(&test_name)?),
                    name: test_name.to_string(),
                    last_ran: Some(latest_result.time_created),
                });
            } else {
                test_runs.push(TestDetails {
                    created: "TBD".to_string(),
                    diffs: None,
                    name: test_name.to_string(),
                    last_ran: Some(latest_result.time_created),
                });
            }
        }
    }
    let mut state_data = STATE_DATA.lock().unwrap();
    state_data.pattern = pattern.to_string();
    state_data.runs = test_runs;
    state_data.state_updated = time::now();
    Ok(())
}

fn get_diffs(test_name: &String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let differences = db::read_differences(&test_name)?;
    let mut diffs = vec![];
    for difference in differences {
        match difference.0.as_ref() {
            "5" => continue,
            "8" => continue,
            _ => (),
        }
        diffs.push(match difference.0.as_ref() {
            "1" => format!("+ Actual code: {}", difference.1).to_string(),
            "2" => format!("- Expected code: {}", difference.1).to_string(),
            "3" => format!("+ Stderr add: {}", difference.1).to_string(),
            "4" => format!("- Stderr remove: {}", difference.1).to_string(),
            //   "5" => format!("= Stderr same: {}", difference.1).to_string(),
            "6" => format!("+ Stdout add: {}", difference.1).to_string(),
            "7" => format!("- Stdout remove: {}", difference.1).to_string(),
            //   "8" => format!("= Stdout same: {}", difference.1).to_string(),
            _ => format!("not yet implemented: {} {}", difference.0, difference.1).to_string(),
        });
    }
    Ok(diffs)
}

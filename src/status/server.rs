use std::net;
use std::sync::Mutex;

use gotham::state::State;

use crate::config;
use crate::db;
use crate::finder;
use crate::status::monitor;
use crate::status::views::status;
use crate::time;

lazy_static! {
    /// initialize static Shared State Data
    pub static ref STATE_DATA: Mutex<StateData> = Mutex::new(StateData {
        pattern: "".to_string(),
        runs: vec![],
        server_started: time::now(),
        state_updated: "".to_string(),
    });
}

/// Shared State Data
#[derive(Debug)]
pub struct StateData {
    /// Test name pattern
    pub pattern: String,
    /// List of test results
    pub runs: Vec<TestDetails>,
    server_started: String,
    /// updated time
    pub state_updated: String,
}

/// Test details
#[derive(Debug, Serialize)]
pub struct TestDetails {
    /// When test was first run
    pub created: String,
    /// Test results differences from last run (if any)
    pub diffs: Option<Vec<String>>,
    /// When test was last run (if more than once)
    pub last_ran: Option<String>,
    /// Test name
    pub name: String,
}

/// start status server
pub fn start(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("server/start");
    let status_port = config.status_port();
    let pattern = config.extract_pattern().to_string();
    set_test_runs(pattern.to_string())?;
    let handles = monitor::launch_monitor(pattern);
    let addr = format!("{}:{}", net::Ipv4Addr::LOCALHOST.to_string(), status_port);

    println!("Listening at {}.  Ctrl-C to terminate server", addr);
    gotham::start(addr, || Ok(serve_status_view)); // loops until Ctrl-C kills process
    for handle in handles {
        handle.join().unwrap();
    }
    Ok(())
}

/// Serve the status view
fn serve_status_view(state: State) -> (State, (mime::Mime, String)) {
    log::info!("server/serve_status_view");
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
    let status_counts = status::StatusCounts {
        fail_count: format!(" {:05}", failed_test_names.len()),
        not_run_count: format!(" {:05}", not_yet_run_test_names.len()),
        pass_count: format!(" {:05}", passed_test_names.len()),
        test_count: format!(" {:05}", state_data.runs.len()),
    };
    let status_flags = status::StatusFlags {
        no_failed_tests: failed_test_names.is_empty(),
        no_not_yet_run_tests: not_yet_run_test_names.is_empty(),
        no_passed_tests: passed_test_names.is_empty(),
    };
    let status_view = status::render(&status::StatusViewContext::new(
        state_data.server_started.clone(),
        state_data.state_updated.clone(),
        status_counts,
        status_flags,
        state_data.pattern.to_string(),
        copied_runs,
    ))
    .unwrap();
    let response_str = format!("<div>{}</div>", status_view);
    (state, (mime::TEXT_HTML, response_str))
}

/// update shared state with test run data
pub fn set_test_runs(pattern: String) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("server/set_test_runs pattern: {}", &pattern);
    let mut test_runs = vec![];
    let test_names = finder::discover(pattern.to_string())?;
    for test_name in &test_names.found {
        let original_result = db::read_original_results(test_name)?;
        let latest_results_row_count = db::count_latest_results(test_name)?;
        if latest_results_row_count == 0 {
            test_runs.push(TestDetails {
                created: original_result.time_created,
                diffs: None,
                name: test_name.to_string(),
                last_ran: None,
            });
        } else {
            let latest_result = db::read_latest_results(test_name)?;
            let difference_count = db::count_differences(test_name)?;
            if difference_count > 0 {
                test_runs.push(TestDetails {
                    created: original_result.time_created,
                    diffs: Some(get_diffs(test_name)?),
                    name: test_name.to_string(),
                    last_ran: Some(latest_result.time_created),
                });
            } else {
                test_runs.push(TestDetails {
                    created: original_result.time_created,
                    diffs: None,
                    name: test_name.to_string(),
                    last_ran: Some(latest_result.time_created),
                });
            }
        }
    }
    let mut state_data = STATE_DATA.lock().unwrap();
    state_data.pattern = pattern;
    state_data.runs = test_runs;
    state_data.state_updated = time::now();
    Ok(())
}

/// get test result differences
fn get_diffs(test_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    log::info!("server/get_diffs test_name {}", &test_name);
    let differences = db::read_differences(test_name)?;
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

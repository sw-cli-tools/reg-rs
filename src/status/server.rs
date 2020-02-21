use std::sync::Mutex;

use gotham::state::State;
use mime;

use crate::config;
use crate::db;
use crate::finder;
use crate::status::views::status;
use crate::time;

#[derive(Debug)]
struct StateData {
    views: Vec<App>, 
}

lazy_static!{
    static ref STATE_DATA: Mutex<StateData> = Mutex::new(
        StateData {
            views: vec![],
        });
}

#[derive(Debug)]
struct App {
    fail_count: u32,
    failed_test_names: Vec<String>,
    no_failed_tests: bool,
    no_not_yet_run_tests: bool,
    no_passed_tests: bool,
    not_yet_run_count: u32,
    not_yet_run_test_names: Vec<String>,
    pass_count: u32,
    passed_test_names: Vec<String>,
    test_count: u32,
    test_pattern: Box<String>,
}

impl App {
    fn new(
        fail_count: u32,
        failed_test_names: Vec<String>,
        no_failed_tests: bool,
        no_not_yet_run_tests: bool,
        no_passed_tests: bool,
        not_yet_run_count: u32,
        not_yet_run_test_names: Vec<String>,
        pass_count: u32,
        passed_test_names: Vec<String>,
        test_count: u32,
        test_pattern: &str) -> Self {
        App {
            fail_count,
            failed_test_names,
            no_failed_tests,
            no_not_yet_run_tests,
            no_passed_tests,
            not_yet_run_count,
            not_yet_run_test_names,
            pass_count,
            passed_test_names,
            test_count,
            test_pattern: Box::new(test_pattern.to_string()),
        }
    }
}

pub fn start(config: &config::Config
) -> Result<(), Box<dyn std::error::Error>> {
    md!(&config);
    let status_port = config.status_port();
    md!(status_port);
    let test_pattern = config.extract_pattern();
    md!(&test_pattern);
    let test_names = finder::discover(&config)?;
    let test_count = *&test_names.found.len() as u32;
    let mut failed_test_names = vec![];
    let mut passed_test_names = vec![];
    let mut not_yet_run_test_names = vec![];
    for test_name in &test_names.found {
        let latest_results_table_count = db::latest_results_table_count(&test_name)?;
        if latest_results_table_count == 0 {
            not_yet_run_test_names.push(format!("{}", &test_name));
        } else {
            let difference_count = db::count_differences(&test_name)?;
            md!(difference_count);
            if difference_count > 0 {
                failed_test_names.push(format!("{}", &test_name));
            } else {
                passed_test_names.push(format!("{}", &test_name));
            }
        }
    }
    let fail_count = failed_test_names.len() as u32;
    let no_failed_tests = 0 == *&failed_test_names.len();
    let not_yet_run_count = not_yet_run_test_names.len() as u32;
    let no_not_yet_run_tests = 0 == *&not_yet_run_test_names.len();
    let pass_count = passed_test_names.len() as u32;
    let no_passed_tests = 0 == *&passed_test_names.len();
    let app = App::new(
        fail_count,
        failed_test_names,
        no_failed_tests,
        no_not_yet_run_tests,
        no_passed_tests,
        not_yet_run_count,
        not_yet_run_test_names,
        pass_count,
        passed_test_names,
        test_count,
        test_pattern);
    {
        let mut state_data = STATE_DATA.lock().unwrap();
        state_data.views.push(app); // mutate state before handling requests
    } // unlock STATE_DATA here, otherwise serve_status_view hangs waiting for lock
    let addr = format!("localhost:{}", status_port);
    println!("Listening at {}.  Ctrl-C to terminate server", addr);
    gotham::start(addr, || Ok(serve_status_view)); // loops until Ctrl-C kills process
    Ok(())
}

fn serve_status_view(state: State) -> (State, (mime::Mime, String)) {
    let view_state = &STATE_DATA.lock().unwrap().views[0];
    let status_view = status::render(&status::StatusViewContext::new(
        view_state.fail_count,
        view_state.failed_test_names.clone(),
        view_state.no_failed_tests,
        view_state.no_not_yet_run_tests,
        view_state.no_passed_tests,
        view_state.not_yet_run_count,
        view_state.not_yet_run_test_names.clone(),
        view_state.pass_count,
        view_state.passed_test_names.clone(),
        time::now(),
        view_state.test_count,
        view_state.test_pattern.to_string(),
    )).unwrap();
    md!(&status_view);
    
    let response_str = format!("<div>{}</div>", status_view);
    (state, (mime::TEXT_HTML, response_str.to_string()))
}

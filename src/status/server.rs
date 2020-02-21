use iron::method::Method;
use iron::prelude::*;
use iron::status::MethodNotAllowed;
use iron::Handler;

use crate::config;
use crate::db;
use crate::finder;
use crate::status::views::status;
use crate::time;

struct App {
    fail_count: u32,
    not_yet_run_count: u32,
    pass_count: u32,
    test_count: u32,
    test_pattern: Box<String>,
}

impl App {
    fn new(
        fail_count: u32,
        not_yet_run_count: u32,
        pass_count: u32,
        test_count: u32,
        test_pattern: &str) -> Self {
        App {
            fail_count,
            not_yet_run_count,
            pass_count,
            test_count,
            test_pattern: Box::new(test_pattern.to_string()),
        }
    }
    fn status(&self, request: &mut Request) -> IronResult<Response> {
        let status_view = status::render(&status::StatusViewContext::new(
            self.fail_count,
            self.not_yet_run_count,
            self.pass_count,
            time::now(),
            self.test_count,
            self.test_pattern.to_string(),
        ))
        .unwrap();
        md!(&status_view);
        Ok(match request.method {
            Method::Get => {
                let mut response = Response::new();
                response.set_mut(status_view).set_mut(iron::status::Ok);
                response
            }
            _ => Response::with((MethodNotAllowed, "")),
        })
    }
}

impl Handler for App {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        match request.url.path().join("/").as_ref() {
            "status" => self.status(request),
            _ => Ok(Response::with((MethodNotAllowed, ""))),
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
    let service = format!("localhost:{}", status_port);
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
    let not_yet_run_count = not_yet_run_test_names.len() as u32;
    let pass_count = passed_test_names.len() as u32;
    let app = App::new(
        fail_count,
        not_yet_run_count,
        pass_count,
        test_count,
        test_pattern);
    println!("Ctrl-C to terminate status server");
    Iron::new(app).http(service).unwrap();
    Ok(())
}

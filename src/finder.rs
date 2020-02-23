use log;
use walkdir::{DirEntry, Error, WalkDir};

#[derive(Debug)]
pub struct TestNames {
    pub found: Vec<String>,
}

fn execute_closure(
    closure_argument: &mut dyn FnMut(&mut Vec<String>, String),
    acc: &mut Vec<String>,
    value: String,
) {
    closure_argument(acc, value);
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn subject(pattern: String) -> Result<TestNames, Error> {
    let mut tests = TestNames { found: vec![] };
    let mut closure_variable = |acc: &mut Vec<String>, val: String| {
        if val.contains(&pattern)
        && val.ends_with(".tdb"){
            md!(&val);
            md!(&pattern);
            acc.push(val);
        }
    };
    let walker = WalkDir::new("data").into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        execute_closure(
            &mut closure_variable,
            &mut tests.found,
            format!("{}", entry?.path().display()),
        );
    }
    Ok(tests)
}

pub fn discover(pattern: String) -> Result<TestNames, Box<dyn std::error::Error>> {
    log::info!("finder/discover pattern {}", &pattern);
    let tests = subject(pattern)?;
    md!(&tests);
    Ok(tests)
}

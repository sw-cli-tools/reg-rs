use std::sync::Mutex;

use crate::dispatch;

/// Run tests sequentially
pub fn run_many_sequential(tests: &[String], dry_run: bool) -> reg_rs_types::error::Result<()> {
    for test in tests {
        dispatch::run_and_diff(test, dry_run)?;
    }
    Ok(())
}

/// Run tests in parallel using scoped threads (one thread per test)
pub fn run_many_parallel(tests: &[String], dry_run: bool) -> reg_rs_types::error::Result<()> {
    let test_count = tests.len();
    eprintln!("running {} tests in parallel", test_count);
    let start = std::time::Instant::now();

    let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());

    std::thread::scope(|s| {
        for test in tests {
            let errors = &errors;
            s.spawn(move || {
                if let Err(e) = dispatch::run_and_diff(test, dry_run) {
                    errors
                        .lock()
                        .expect("mutex poisoned collecting test errors")
                        .push(format!("{}: {}", test, e));
                }
            });
        }
    });

    let elapsed = start.elapsed();
    eprintln!(
        "parallel run complete: {} tests in {:.2}s",
        test_count,
        elapsed.as_secs_f64()
    );

    let errors = errors
        .into_inner()
        .expect("mutex poisoned extracting test errors");
    if errors.is_empty() {
        Ok(())
    } else {
        Err(reg_rs_types::error::RegError::Other(format!(
            "{} test(s) failed:\n  {}",
            errors.len(),
            errors.join("\n  ")
        )))
    }
}

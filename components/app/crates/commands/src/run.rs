use reg_rs_config::config::Config;
use reg_rs_store::db_ops;
use reg_rs_store_rgt::rgt_util;
use reg_rs_types::error::Result;
use reg_rs_types::types::RegressionType;

/// Run tests and return the number of failures
pub fn run(config: &Config) -> Result<u32> {
    log::info!("command/run");
    run_tests(config)?;
    let quiet = config.is_quiet();
    let verbosity = config.verbosity_level();
    let pattern = config.extract_pattern().to_string();
    let tests = reg_rs_discover::finder::discover(pattern)?;
    let total = tests.found.len() as u32;
    let mut fail_count = 0u32;
    let mut failed_paths = vec![];
    for test_path in &tests.found {
        let db_path = rgt_util::db_path(test_path);
        let latest_count = db_ops::count_latest_results(&db_path)?;
        if latest_count > 0 && db_ops::count_differences(&db_path)? > 0 {
            fail_count += 1;
            failed_paths.push(test_path.clone());
        }
    }
    if quiet {
        return Ok(fail_count);
    }
    let pass_count = total - fail_count;
    if fail_count > 0 {
        eprintln!(
            "{} passed, {} failed (of {} total)",
            pass_count, fail_count, total
        );
        for test_path in &failed_paths {
            let name = crate::utils::format_test_name(test_path);
            eprintln!("  FAIL: {}", name);
        }
        if verbosity > 0 {
            run_show_failure_details(&failed_paths, verbosity)?;
        }
    } else {
        eprintln!("{} passed (of {} total)", pass_count, total);
    }
    Ok(fail_count)
}

fn run_tests(config: &Config) -> Result<()> {
    let pattern = config.extract_pattern().to_string();
    let result = reg_rs_discover::finder::discover(pattern.clone())?;
    if result.found.is_empty() {
        eprintln!(
            "warning: no tests matched pattern '{}' in {}",
            pattern,
            result.data_dir.display()
        );
        return Ok(());
    }
    if config.is_parallel() {
        reg_rs_runner::runner_parallel::run_many_parallel(&result.found, config.is_dry_run())
    } else {
        reg_rs_runner::runner_parallel::run_many_sequential(&result.found, config.is_dry_run())
    }
}

fn run_show_failure_details(failed_paths: &[String], verbosity: u8) -> Result<()> {
    for test_path in failed_paths {
        let db_path = rgt_util::db_path(test_path);
        let name = crate::utils::format_test_name(test_path);
        let differences = db_ops::read_differences(&db_path)?;
        let same_count =
            db_ops::difference_count_by_type(&db_path, RegressionType::StderrSame as u8)?
                + db_ops::difference_count_by_type(&db_path, RegressionType::StdoutSame as u8)?;
        let diff_count = differences.len() as u32 - same_count;

        let mut types = vec![];
        if db_ops::difference_count_by_type(&db_path, RegressionType::ActualCode as u8)? > 0 {
            types.push("exit_code");
        }
        if db_ops::difference_count_by_type(&db_path, RegressionType::StderrAdd as u8)? > 0
            || db_ops::difference_count_by_type(&db_path, RegressionType::StderrRemove as u8)? > 0
        {
            types.push("stderr");
        }
        if db_ops::difference_count_by_type(&db_path, RegressionType::StdoutAdd as u8)? > 0
            || db_ops::difference_count_by_type(&db_path, RegressionType::StdoutRemove as u8)? > 0
        {
            types.push("stdout");
        }

        eprintln!();
        let type_str = if types.is_empty() {
            String::new()
        } else {
            format!(" ({})", types.join(", "))
        };
        eprintln!("  {} — {} difference(s){}", name, diff_count, type_str);

        if let Ok(Some(desc)) = db_ops::read_metadata(&db_path, "desc") {
            eprintln!("    desc:    {}", desc);
        }
        if let Ok(Some(expects)) = db_ops::read_metadata(&db_path, "expects") {
            eprintln!("    expects: {}", expects);
        }

        if verbosity > 1 {
            for difference in &differences {
                if let Some(label) = RegressionType::display_label(&difference.0) {
                    eprintln!("    [{}] {}", label, difference.1);
                }
            }
        }
    }
    Ok(())
}

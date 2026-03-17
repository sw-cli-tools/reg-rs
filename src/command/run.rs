use crate::config;
use crate::db;
use crate::diff;
use crate::finder;

/// Run tests and return the number of failures
pub fn run(config: &config::Config) -> crate::error::Result<u32> {
    log::info!("command/run");
    crate::runner::run_many(config)?;
    let quiet = config.is_quiet();
    let verbosity = config.verbosity_level();
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern)?;
    let total = tests.found.len() as u32;
    let mut fail_count = 0u32;
    let mut failed_paths = vec![];
    for test_path in &tests.found {
        let db = crate::db_path(test_path);
        let latest_count = db::count_latest_results(&db)?;
        if latest_count > 0 && db::count_differences(&db)? > 0 {
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
            let name = super::utils::format_test_name(test_path);
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

fn run_show_failure_details(failed_paths: &[String], verbosity: u8) -> crate::error::Result<()> {
    for test_path in failed_paths {
        let db = crate::db_path(test_path);
        let name = super::utils::format_test_name(test_path);
        let differences = db::read_differences(&db)?;
        let same_count = db::difference_count_by_type(&db, diff::RegressionType::StderrSame as u8)?
            + db::difference_count_by_type(&db, diff::RegressionType::StdoutSame as u8)?;
        let diff_count = differences.len() as u32 - same_count;

        let mut types = vec![];
        if db::difference_count_by_type(&db, diff::RegressionType::ActualCode as u8)? > 0 {
            types.push("exit_code");
        }
        if db::difference_count_by_type(&db, diff::RegressionType::StderrAdd as u8)? > 0
            || db::difference_count_by_type(&db, diff::RegressionType::StderrRemove as u8)? > 0
        {
            types.push("stderr");
        }
        if db::difference_count_by_type(&db, diff::RegressionType::StdoutAdd as u8)? > 0
            || db::difference_count_by_type(&db, diff::RegressionType::StdoutRemove as u8)? > 0
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

        if let Ok(Some(desc)) = db::read_metadata(&db, "desc") {
            eprintln!("    desc:    {}", desc);
        }
        if let Ok(Some(expects)) = db::read_metadata(&db, "expects") {
            eprintln!("    expects: {}", expects);
        }

        if verbosity > 1 {
            for difference in &differences {
                if let Some(label) = diff::RegressionType::display_label(&difference.0) {
                    eprintln!("    [{}] {}", label, difference.1);
                }
            }
        }
    }
    Ok(())
}

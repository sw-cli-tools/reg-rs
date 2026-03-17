use reg_rs_config::config::Config;
use reg_rs_store::db;
use reg_rs_store::db_ops;
use reg_rs_store::queries;
use reg_rs_store_rgt::rgt;
use reg_rs_store_rgt::rgt_util;
use reg_rs_types::constants::RGT_EXTENSION;
use reg_rs_types::error::Result;
use reg_rs_types::types::TestResults;

/// Rebase tests to accept latest output as baseline
pub fn rebase(config: &Config) -> Result<()> {
    log::info!("command/rebase");
    let pattern = config.extract_pattern().to_string();
    let tests = reg_rs_discover::finder::discover(pattern.clone())?;
    if tests.found.is_empty() {
        eprintln!(
            "no tests matched pattern '{}' in {}",
            pattern,
            tests.data_dir.display()
        );
        return Ok(());
    }
    let mut rebased = 0;
    for test_path in &tests.found {
        if rebase_one(test_path)? {
            rebased += 1;
        }
    }
    eprintln!("{} test(s) rebased", rebased);
    log::info!("command/rebase done");
    Ok(())
}

/// Rebase a single test, returning true if rebased.
fn rebase_one(test_path: &str) -> Result<bool> {
    let is_rgt = test_path.ends_with(&format!(".{}", RGT_EXTENSION));
    let tdb_path = if is_rgt {
        rgt_util::tdb_path_for_rgt(test_path)
    } else {
        test_path.to_string()
    };

    let latest_count = db_ops::count_latest_results(&tdb_path)?;
    if latest_count == 0 {
        eprintln!(
            "skip: {} (no latest results — run the test first)",
            test_path
        );
        return Ok(false);
    }

    let latest = db::read_latest_results(&tdb_path)?;

    if is_rgt {
        rgt::write_baseline(test_path, &latest.stdout, &latest.stderr)?;
        let spec = rgt::parse_rgt(test_path)?;
        if spec.exit_code.is_some() {
            let updated = rgt::RgtSpec {
                exit_code: Some(latest.exit_code),
                ..spec
            };
            rgt::write_rgt(test_path, &updated)?;
        }
    } else {
        db_ops::reset_differences(&tdb_path)?;
        db::store_results(
            &tdb_path,
            &TestResults {
                name: latest.name,
                command: latest.command,
                time_created: latest.time_created,
                exit_code: latest.exit_code,
                stdout: latest.stdout,
                stderr: latest.stderr,
            },
            queries::StatementContext::original(),
        )?;
    }

    db_ops::clear_differences(&tdb_path)?;
    eprintln!("rebased: {}", test_path);
    Ok(true)
}

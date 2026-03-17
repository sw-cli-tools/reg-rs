use reg_rs_config::config::Config;
use reg_rs_store::db;
use reg_rs_store_rgt::rgt_util;
use reg_rs_types::constants::{LOCK_EXTENSION, RGT_EXTENSION};
use reg_rs_types::error::Result;

/// Remove tests matching a pattern
pub fn remove(config: &Config) -> Result<()> {
    log::info!("command/remove");
    let pattern = config.extract_pattern().to_string();
    let tests = reg_rs_discover::finder::discover(pattern.clone())?;
    log::debug!("remove tests: {:?}", &tests);
    if tests.found.is_empty() {
        eprintln!(
            "warning: no tests matched pattern '{}' in {}",
            pattern,
            tests.data_dir.display()
        );
        return Ok(());
    }
    for test in &tests.found {
        let is_rgt = test.ends_with(&format!(".{}", RGT_EXTENSION));
        if is_rgt {
            remove_rgt_files(test)?;
        } else {
            remove_tdb_files(test)?;
        }
    }
    log::info!("command/remove done");
    Ok(())
}

/// Remove all files associated with an `.rgt` test.
fn remove_rgt_files(test: &str) -> Result<()> {
    let _ = std::fs::remove_file(test);
    let out_path =
        std::path::Path::new(test).with_extension(reg_rs_types::constants::OUT_EXTENSION);
    let _ = std::fs::remove_file(&out_path);
    let err_path =
        std::path::Path::new(test).with_extension(reg_rs_types::constants::ERR_EXTENSION);
    let _ = std::fs::remove_file(&err_path);
    let tdb_path = rgt_util::tdb_path_for_rgt(test);
    let _ = db::drop_all_results(&tdb_path);
    let _ = std::fs::remove_file(&tdb_path);
    let lock_path = format!("{}.{}", tdb_path, LOCK_EXTENSION);
    let _ = std::fs::remove_file(&lock_path);
    let rgt_lock = format!("{}.{}", test, LOCK_EXTENSION);
    let _ = std::fs::remove_file(&rgt_lock);
    Ok(())
}

/// Remove all files associated with a `.tdb` test.
fn remove_tdb_files(test: &str) -> Result<()> {
    db::drop_all_results(test)?;
    if let Err(e) = std::fs::remove_file(test) {
        log::debug!("could not remove {}: {}", test, e);
    }
    let lock_path = format!("{}.{}", test, LOCK_EXTENSION);
    if let Err(e) = std::fs::remove_file(&lock_path) {
        log::debug!("could not remove {}: {}", lock_path, e);
    }
    Ok(())
}

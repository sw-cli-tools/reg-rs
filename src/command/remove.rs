use crate::config;
use crate::db;
use crate::finder;
use crate::rgt;

/// Remove tests matching a pattern
pub fn remove(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/remove");
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern.clone())?;
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
        let is_rgt = test.ends_with(&format!(".{}", rgt::RGT_EXTENSION));
        if is_rgt {
            let _ = std::fs::remove_file(test);
            let out_path = std::path::Path::new(test).with_extension(rgt::OUT_EXTENSION);
            let _ = std::fs::remove_file(&out_path);
            let err_path = std::path::Path::new(test).with_extension(rgt::ERR_EXTENSION);
            let _ = std::fs::remove_file(&err_path);
            let tdb_path = rgt::tdb_path_for_rgt(test);
            let _ = db::drop_all_results(&tdb_path);
            let _ = std::fs::remove_file(&tdb_path);
            let lock_path = format!("{}.{}", tdb_path, crate::LOCK_EXTENSION);
            let _ = std::fs::remove_file(&lock_path);
            let rgt_lock = format!("{}.{}", test, crate::LOCK_EXTENSION);
            let _ = std::fs::remove_file(&rgt_lock);
        } else {
            db::drop_all_results(test)?;
            if let Err(e) = std::fs::remove_file(test) {
                log::debug!("could not remove {}: {}", test, e);
            }
            let lock_path = format!("{}.{}", test, crate::LOCK_EXTENSION);
            if let Err(e) = std::fs::remove_file(&lock_path) {
                log::debug!("could not remove {}: {}", lock_path, e);
            }
        }
    }
    log::info!("command/remove done");
    Ok(())
}

use reg_rs_config::config::Config;
use reg_rs_store::db;
use reg_rs_store::db_ops;
use reg_rs_store_rgt::rgt;
use reg_rs_types::constants::{DIFF_MODE_KEY, PREPROCESS_KEY, RGT_EXTENSION};
use reg_rs_types::error::Result;

const META_DESC: &str = "desc";
const META_EXPECTS: &str = "expects";
const META_FLAKY_NOTE: &str = "flaky_note";

/// Migrate .tdb tests to .rgt format
pub fn migrate(config: &Config) -> Result<()> {
    log::info!("command/migrate");
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
    let mut migrated = 0;
    for test_path in &tests.found {
        if test_path.ends_with(&format!(".{}", RGT_EXTENSION)) {
            eprintln!("skip: {} (already .rgt)", test_path);
            continue;
        }
        migrate_one(test_path)?;
        migrated += 1;
    }
    eprintln!("{} test(s) migrated to .rgt format", migrated);
    log::info!("command/migrate done");
    Ok(())
}

/// Migrate a single `.tdb` test to `.rgt` format.
fn migrate_one(tdb_path: &str) -> Result<()> {
    let original = db::read_original_results(tdb_path)?;
    let rgt_path = std::path::Path::new(tdb_path)
        .with_extension(RGT_EXTENSION)
        .to_string_lossy()
        .to_string();

    if std::path::Path::new(&rgt_path).exists() {
        eprintln!("skip: {} (.rgt already exists)", tdb_path);
        return Ok(());
    }

    let timeout = db_ops::read_metadata(tdb_path, "timeout")?.and_then(|s| s.parse::<u64>().ok());
    let preprocess = db_ops::read_metadata(tdb_path, PREPROCESS_KEY)?;
    let diff_mode = db_ops::read_metadata(tdb_path, DIFF_MODE_KEY)?;
    let desc = db_ops::read_metadata(tdb_path, META_DESC)?;
    let expects = db_ops::read_metadata(tdb_path, META_EXPECTS)?;
    let flaky_note = db_ops::read_metadata(tdb_path, META_FLAKY_NOTE)?;

    let spec = rgt::RgtSpec {
        command: original.command,
        timeout,
        preprocess,
        diff_mode,
        exit_code: Some(original.exit_code),
        desc,
        expects,
        flaky_note,
    };

    rgt::write_rgt(&rgt_path, &spec)?;
    rgt::write_baseline(&rgt_path, &original.stdout, &original.stderr)?;
    eprintln!("migrated: {} -> {}", tdb_path, rgt_path);
    Ok(())
}

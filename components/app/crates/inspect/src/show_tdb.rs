use reg_rs_store::db;
use reg_rs_store::db_ops;
use reg_rs_types::constants::{DIFF_MODE_KEY, PREPROCESS_KEY};
use reg_rs_types::error::Result;

use crate::show::{print_baselines, show_latest_and_diffs};

const META_DESC: &str = "desc";
const META_EXPECTS: &str = "expects";
const META_FLAKY_NOTE: &str = "flaky_note";

/// Show details for a single `.tdb` test.
pub(crate) fn show_one_tdb(
    test_path: &str,
    name: &str,
    status: &str,
    verbosity: u8,
    latest_count: u32,
    diff_count: u32,
) -> Result<()> {
    let original = db::read_original_results(test_path)?;

    println!("=== {name} ({status}) ===");
    println!("command:  {}", original.command);
    println!("created:  {}", original.time_created);
    println!("exit:     {}", original.exit_code);
    print_tdb_metadata(test_path);

    if verbosity >= 1 {
        print_baselines(&original.stdout, &original.stderr);
    }

    if verbosity >= 2 && latest_count > 0 {
        show_latest_and_diffs(test_path, diff_count)?;
    }

    Ok(())
}

/// Print metadata fields stored in a `.tdb` database.
fn print_tdb_metadata(test_path: &str) {
    for (key, label) in [
        (META_DESC, "desc"),
        (META_EXPECTS, "expects"),
        (META_FLAKY_NOTE, "flaky"),
        (PREPROCESS_KEY, "preprocess"),
        (DIFF_MODE_KEY, "diff_mode"),
        ("timeout", "timeout"),
    ] {
        if let Ok(Some(val)) = db_ops::read_metadata(test_path, key) {
            println!("{:<10}{}", format!("{}:", label), val);
        }
    }
}

use reg_rs_store_rgt::rgt;
use reg_rs_types::error::Result;

use crate::show::{print_baselines, print_optional_field, show_latest_and_diffs};

/// Show details for a single `.rgt` test.
pub(crate) fn show_one_rgt(
    rgt_path: &str,
    name: &str,
    status: &str,
    verbosity: u8,
    tdb_path: &str,
    latest_count: u32,
    diff_count: u32,
) -> Result<()> {
    let spec = rgt::parse_rgt(rgt_path)?;

    println!("=== {name} ({status}) ===");
    println!("format:   .rgt");
    println!("command:  {}", spec.command);
    print_optional_field("exit", &spec.exit_code.map(|c| c.to_string()));
    print_optional_field("desc", &spec.desc);
    print_optional_field("expects", &spec.expects);
    print_optional_field("flaky", &spec.flaky_note);
    print_optional_field("preprocess", &spec.preprocess);
    print_optional_field("diff_mode", &spec.diff_mode);
    print_optional_field("timeout", &spec.timeout.map(|t| format!("{t}s")));

    if verbosity >= 1 {
        let baseline_stdout = rgt::read_baseline_stdout(rgt_path)?;
        let baseline_stderr = rgt::read_baseline_stderr(rgt_path)?;
        print_baselines(&baseline_stdout, &baseline_stderr);
    }

    if verbosity >= 2 && latest_count > 0 {
        show_latest_and_diffs(tdb_path, diff_count)?;
    }

    Ok(())
}

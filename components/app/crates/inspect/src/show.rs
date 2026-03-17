use reg_rs_config::config::Config;
use reg_rs_store::db;
use reg_rs_store::db_ops;
use reg_rs_store_rgt::rgt_util;
use reg_rs_types::constants::RGT_EXTENSION;
use reg_rs_types::error::Result;
use reg_rs_types::types::RegressionType;

/// Show detailed information about tests
pub fn show(config: &Config) -> Result<()> {
    log::info!("command/show");
    let pattern = config.extract_pattern().to_string();
    let verbosity = config.verbosity_level();
    let tests = reg_rs_discover::finder::discover(pattern.clone())?;
    if tests.found.is_empty() {
        eprintln!(
            "no tests matched pattern '{}' in {}",
            pattern,
            tests.data_dir.display()
        );
        return Ok(());
    }
    for (i, test_path) in tests.found.iter().enumerate() {
        if i > 0 {
            println!();
        }
        show_one_test(test_path, verbosity)?;
    }
    log::info!("command/show done");
    Ok(())
}

/// Dispatch to the appropriate show handler based on file type.
fn show_one_test(test_path: &str, verbosity: u8) -> Result<()> {
    let is_rgt = test_path.ends_with(&format!(".{}", RGT_EXTENSION));
    let name = format_test_name(test_path);
    let tdb_path = if is_rgt {
        rgt_util::tdb_path_for_rgt(test_path)
    } else {
        test_path.to_string()
    };

    let latest_count = db_ops::count_latest_results(&tdb_path)?;
    let diff_count = db_ops::count_differences(&tdb_path)?;

    let status = if latest_count == 0 {
        "pending"
    } else if diff_count > 0 {
        "FAIL"
    } else {
        "PASS"
    };

    if is_rgt {
        crate::show_rgt::show_one_rgt(
            test_path,
            &name,
            status,
            verbosity,
            &tdb_path,
            latest_count,
            diff_count,
        )
    } else {
        crate::show_tdb::show_one_tdb(
            test_path,
            &name,
            status,
            verbosity,
            latest_count,
            diff_count,
        )
    }
}

/// Print a labeled field only when the value is present.
pub(crate) fn print_optional_field(label: &str, value: &Option<String>) {
    if let Some(v) = value {
        println!("{:<10}{}", format!("{}:", label), v);
    }
}

/// Print baseline stdout/stderr sections.
pub(crate) fn print_baselines(stdout: &str, stderr: &str) {
    println!("\n--- baseline stdout ---");
    print_content_or_empty(stdout);
    if !stderr.is_empty() {
        println!("--- baseline stderr ---");
        print_terminated(stderr);
    }
}

/// Print content, showing "(empty)" for blank strings, ensuring a trailing newline.
pub(crate) fn print_content_or_empty(content: &str) {
    if content.is_empty() {
        println!("(empty)");
    } else {
        print_terminated(content);
    }
}

/// Print text, appending a newline if the text doesn't already end with one.
pub(crate) fn print_terminated(text: &str) {
    print!("{}", text);
    if !text.ends_with('\n') {
        println!();
    }
}

/// Show latest results and differences from the `.tdb` cache.
pub(crate) fn show_latest_and_diffs(tdb_path: &str, diff_count: u32) -> Result<()> {
    let latest = db::read_latest_results(tdb_path)?;
    println!("\n--- latest stdout ---");
    print_content_or_empty(&latest.stdout);
    if !latest.stderr.is_empty() {
        println!("--- latest stderr ---");
        print_terminated(&latest.stderr);
    }
    println!("--- latest exit: {} ---", latest.exit_code);

    if diff_count > 0 {
        let diffs = db_ops::read_differences(tdb_path)?;
        println!("\n--- differences ({}) ---", diffs.len());
        for (type_code, chunk) in &diffs {
            let label = RegressionType::display_label(type_code).unwrap_or("unknown");
            println!("[{}] {}", label, chunk);
        }
    }
    Ok(())
}

/// Format a test path as a display name (file stem only).
fn format_test_name(test_path: &str) -> String {
    std::path::Path::new(test_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

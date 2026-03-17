use crate::config;
use crate::db;
use crate::finder;
use crate::rgt;

use super::utils::format_test_name;

/// Show detailed information about tests
pub fn show(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/show");
    let pattern = config.extract_pattern().to_string();
    let verbosity = config.verbosity_level();
    let tests = finder::discover(pattern.clone())?;
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

fn show_one_test(test_path: &str, verbosity: u8) -> crate::error::Result<()> {
    let is_rgt = test_path.ends_with(&format!(".{}", rgt::RGT_EXTENSION));
    let name = format_test_name(test_path);
    let tdb_path = if is_rgt {
        rgt::tdb_path_for_rgt(test_path)
    } else {
        test_path.to_string()
    };

    let latest_count = db::count_latest_results(&tdb_path)?;
    let diff_count = db::count_differences(&tdb_path)?;

    let status = if latest_count == 0 {
        "pending"
    } else if diff_count > 0 {
        "FAIL"
    } else {
        "PASS"
    };

    if is_rgt {
        show_one_rgt(
            test_path,
            &name,
            status,
            verbosity,
            &tdb_path,
            latest_count,
            diff_count,
        )
    } else {
        show_one_tdb(
            test_path,
            &name,
            status,
            verbosity,
            latest_count,
            diff_count,
        )
    }
}

fn show_one_rgt(
    rgt_path: &str,
    name: &str,
    status: &str,
    verbosity: u8,
    tdb_path: &str,
    latest_count: u32,
    diff_count: u32,
) -> crate::error::Result<()> {
    let spec = rgt::parse_rgt(rgt_path)?;

    println!("=== {} ({}) ===", name, status);
    println!("format:   .rgt");
    println!("command:  {}", spec.command);
    if let Some(exit_code) = spec.exit_code {
        println!("exit:     {}", exit_code);
    }
    if let Some(ref d) = spec.desc {
        println!("desc:     {}", d);
    }
    if let Some(ref e) = spec.expects {
        println!("expects:  {}", e);
    }
    if let Some(ref f) = spec.flaky_note {
        println!("flaky:    {}", f);
    }
    if let Some(ref p) = spec.preprocess {
        println!("preprocess: {}", p);
    }
    if let Some(ref dm) = spec.diff_mode {
        println!("diff_mode: {}", dm);
    }
    if let Some(t) = spec.timeout {
        println!("timeout:  {}s", t);
    }

    if verbosity >= 1 {
        let baseline_stdout = rgt::read_baseline_stdout(rgt_path)?;
        let baseline_stderr = rgt::read_baseline_stderr(rgt_path)?;
        println!("\n--- baseline stdout ---");
        if baseline_stdout.is_empty() {
            println!("(empty)");
        } else {
            print!("{}", baseline_stdout);
            if !baseline_stdout.ends_with('\n') {
                println!();
            }
        }
        if !baseline_stderr.is_empty() {
            println!("--- baseline stderr ---");
            print!("{}", baseline_stderr);
            if !baseline_stderr.ends_with('\n') {
                println!();
            }
        }
    }

    if verbosity >= 2 && latest_count > 0 {
        show_latest_and_diffs(tdb_path, diff_count)?;
    }

    Ok(())
}

const META_DESC: &str = "desc";
const META_EXPECTS: &str = "expects";
const META_FLAKY_NOTE: &str = "flaky_note";

fn show_one_tdb(
    test_path: &str,
    name: &str,
    status: &str,
    verbosity: u8,
    latest_count: u32,
    diff_count: u32,
) -> crate::error::Result<()> {
    let original = db::read_original_results(test_path)?;

    println!("=== {} ({}) ===", name, status);
    println!("command:  {}", original.command);
    println!("created:  {}", original.time_created);
    println!("exit:     {}", original.exit_code);

    for (key, label) in [
        (META_DESC, "desc"),
        (META_EXPECTS, "expects"),
        (META_FLAKY_NOTE, "flaky"),
        (crate::preprocess::PREPROCESS_KEY, "preprocess"),
        (crate::normalize::DIFF_MODE_KEY, "diff_mode"),
        ("timeout", "timeout"),
    ] {
        if let Ok(Some(val)) = db::read_metadata(test_path, key) {
            println!("{:<10}{}", format!("{}:", label), val);
        }
    }

    if verbosity >= 1 {
        println!("\n--- baseline stdout ---");
        if original.stdout.is_empty() {
            println!("(empty)");
        } else {
            print!("{}", original.stdout);
            if !original.stdout.ends_with('\n') {
                println!();
            }
        }
        if !original.stderr.is_empty() {
            println!("--- baseline stderr ---");
            print!("{}", original.stderr);
            if !original.stderr.ends_with('\n') {
                println!();
            }
        }
    }

    if verbosity >= 2 && latest_count > 0 {
        show_latest_and_diffs(test_path, diff_count)?;
    }

    Ok(())
}

fn show_latest_and_diffs(tdb_path: &str, diff_count: u32) -> crate::error::Result<()> {
    let latest = db::read_latest_results(tdb_path)?;
    println!("\n--- latest stdout ---");
    if latest.stdout.is_empty() {
        println!("(empty)");
    } else {
        print!("{}", latest.stdout);
        if !latest.stdout.ends_with('\n') {
            println!();
        }
    }
    if !latest.stderr.is_empty() {
        println!("--- latest stderr ---");
        print!("{}", latest.stderr);
        if !latest.stderr.ends_with('\n') {
            println!();
        }
    }
    println!("--- latest exit: {} ---", latest.exit_code);

    if diff_count > 0 {
        let diffs = db::read_differences(tdb_path)?;
        println!("\n--- differences ({}) ---", diffs.len());
        for (type_code, chunk) in &diffs {
            let label = crate::diff::RegressionType::display_label(type_code).unwrap_or("unknown");
            println!("[{}] {}", label, chunk);
        }
    }
    Ok(())
}

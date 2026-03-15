use crate::ai;
use crate::config;
use crate::db;
use crate::finder;
use crate::queries;
use crate::reporters::generate_reports;
use crate::runner;
use crate::status;

/// Create a test result.
///
/// If the test path is just a filename (no directory separators), it is
/// placed in the default data directory (`~/.local/reg-rs/`). The `.tdb`
/// extension is appended automatically if missing.
///
/// Supports two modes:
/// - `--command`: use the provided shell command directly
/// - `--describe`: use AI to generate a command from a natural language description
pub fn create_original(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/create_original");
    log::debug!("create_original config: {:?}", &config);

    let (test, command) = if let Some(tc) = config.extract_test_and_command() {
        tc
    } else if let Some((test, description)) = config.extract_test_and_describe() {
        let context = gather_context(config)?;
        let existing = gather_existing_test_commands();
        let command = ai::generate_command(&description, context.as_deref(), &existing)?;
        eprintln!("AI generated command: {}", &command);
        eprint!("Proceed? [y/n] ");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| crate::error::RegError::Other(format!("Failed to read input: {}", e)))?;
        if !input.trim().eq_ignore_ascii_case("y") {
            eprintln!("Aborted.");
            return Ok(());
        }
        (test, command)
    } else {
        return Ok(());
    };

    let timeout_secs = config.extract_timeout();
    let rgt_path = resolve_test_path(&test);
    let tdb_path = crate::rgt::tdb_path_for_rgt(&rgt_path);
    if let Some(test_result) =
        runner::run_one_timeout(&tdb_path, &command, false, timeout_secs)?
    {
        // Write .rgt spec + .out/.err baselines
        let preprocess = config.extract_preprocess();
        let diff_mode = config.extract_diff_mode().filter(|m| m != "text");
        let (desc, expects, flaky_note) = config.extract_doc_metadata();
        let spec = crate::rgt::RgtSpec {
            command,
            timeout: if timeout_secs != 300 {
                Some(timeout_secs)
            } else {
                None
            },
            preprocess: preprocess.clone(),
            diff_mode: diff_mode.clone(),
            exit_code: Some(test_result.exit_code),
            desc,
            expects,
            flaky_note,
        };
        crate::rgt::write_rgt(&rgt_path, &spec)?;
        crate::rgt::write_baseline(&rgt_path, &test_result.stdout, &test_result.stderr)?;

        // Also populate .tdb cache so run/report work immediately
        db::reset_differences(&tdb_path)?;
        db::reset_latest_results(&tdb_path)?;
        db::store_results(
            &tdb_path,
            &test_result,
            queries::StatementContext::original(),
        )?;
        if let Some(ref pp) = preprocess {
            db::store_metadata(&tdb_path, crate::preprocess::PREPROCESS_KEY, pp)?;
        }
        if let Some(ref dm) = diff_mode {
            db::store_metadata(&tdb_path, crate::normalize::DIFF_MODE_KEY, dm)?;
        }
        if timeout_secs != 300 {
            db::store_metadata(&tdb_path, "timeout", &timeout_secs.to_string())?;
        }
        store_doc_metadata(config, &tdb_path)?;
    }
    Ok(())
}

/// Resolve a test path: if it has no directory component, place it in the
/// data directory. Append `.rgt` extension if missing.
fn resolve_test_path(test: &str) -> String {
    let path = std::path::Path::new(test);
    let mut resolved = if path.parent().is_some_and(|p| p != std::path::Path::new("")) {
        // Has a directory component - use as-is
        path.to_path_buf()
    } else {
        // Just a filename - put it in the data directory
        crate::data_dir().join(test)
    };
    // Strip existing .tdb or .rgt extension, then add .rgt
    if resolved
        .extension()
        .is_some_and(|ext| ext == crate::TDB_EXTENSION || ext == crate::rgt::RGT_EXTENSION)
    {
        resolved.set_extension("");
    }
    let mut name = resolved.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}", crate::rgt::RGT_EXTENSION));
    resolved.set_file_name(name);
    resolved.to_string_lossy().to_string()
}

/// Update a test result
/// Run tests and return the number of failures (tests with differences).
pub fn update_latest(config: &config::Config) -> crate::error::Result<u32> {
    log::info!("command/update_latest");
    runner::run_many(config)?;
    let quiet = config.is_quiet();
    let verbosity = config.verbosity_level();
    // Count failures across all matched tests
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
            let name = std::path::Path::new(test_path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| test_path.clone());
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

/// Show failure details for `run -v` and `run -vv`.
///
/// - verbosity 1: difference counts and types per test
/// - verbosity 2+: full diff output per test
fn run_show_failure_details(
    failed_paths: &[String],
    verbosity: u8,
) -> crate::error::Result<()> {
    use crate::diff;
    for test_path in failed_paths {
        let db = crate::db_path(test_path);
        let name = std::path::Path::new(test_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| test_path.clone());
        let differences = db::read_differences(&db)?;
        let same_count =
            db::difference_count_by_type(&db, diff::RegressionType::StderrSame as u8)?
                + db::difference_count_by_type(&db, diff::RegressionType::StdoutSame as u8)?;
        let diff_count = differences.len() as u32 - same_count;

        // Collect which types of differences exist
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

        // Show doc metadata if present
        if let Ok(Some(desc)) = db::read_metadata(&db, "desc") {
            eprintln!("    desc:    {}", desc);
        }
        if let Ok(Some(expects)) = db::read_metadata(&db, "expects") {
            eprintln!("    expects: {}", expects);
        }

        // -vv: show full diffs
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

/// remove test results
pub fn remove_all(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/remove_all");
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern.clone())?;
    log::debug!("remove_all tests: {:?}", &tests);
    if tests.found.is_empty() {
        eprintln!(
            "warning: no tests matched pattern '{}' in {}",
            pattern,
            tests.data_dir.display()
        );
        return Ok(());
    }
    for test in &tests.found {
        let is_rgt = test.ends_with(&format!(".{}", crate::rgt::RGT_EXTENSION));
        if is_rgt {
            // Remove .rgt spec, .out baseline, .err baseline
            let _ = std::fs::remove_file(test);
            let out_path =
                std::path::Path::new(test).with_extension(crate::rgt::OUT_EXTENSION);
            let _ = std::fs::remove_file(&out_path);
            let err_path =
                std::path::Path::new(test).with_extension(crate::rgt::ERR_EXTENSION);
            let _ = std::fs::remove_file(&err_path);
            // Also remove .tdb cache and its lock
            let tdb_path = crate::rgt::tdb_path_for_rgt(test);
            let _ = db::drop_all_results(&tdb_path);
            let _ = std::fs::remove_file(&tdb_path);
            let lock_path = format!("{}.{}", tdb_path, crate::LOCK_EXTENSION);
            let _ = std::fs::remove_file(&lock_path);
            // Remove .rgt.lock if present
            let rgt_lock = format!("{}.{}", test, crate::LOCK_EXTENSION);
            let _ = std::fs::remove_file(&rgt_lock);
        } else {
            db::drop_all_results(test)?;
            // Clean up the .tdb file and its .lock file
            if let Err(e) = std::fs::remove_file(test) {
                log::debug!("could not remove {}: {}", test, e);
            }
            let lock_path = format!("{}.{}", test, crate::LOCK_EXTENSION);
            if let Err(e) = std::fs::remove_file(&lock_path) {
                log::debug!("could not remove {}: {}", lock_path, e);
            }
        }
    }
    log::info!("command/remove_all done");
    Ok(())
}

/// List tests matching a pattern with name, command, and status.
pub fn list_tests(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/list_tests");
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern.clone())?;
    if tests.found.is_empty() {
        eprintln!(
            "no tests matched pattern '{}' in {}",
            pattern,
            tests.data_dir.display()
        );
        return Ok(());
    }
    for test_path in &tests.found {
        let (command, tdb_path) = read_test_command(test_path)?;
        let latest_count = db::count_latest_results(&tdb_path)?;
        let status = if latest_count == 0 {
            "pending"
        } else {
            let diff_count = db::count_differences(&tdb_path)?;
            if diff_count > 0 { "FAIL" } else { "PASS" }
        };
        // Extract just the test name from the path (strip directory and extension)
        let name = std::path::Path::new(test_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        // Truncate long commands for display
        let cmd_display = if command.len() > 60 {
            format!("{}...", &command[..57])
        } else {
            command
        };
        println!("{:<7} {:<30} {}", status, name, cmd_display);
    }
    println!(
        "---\n{} test(s) matched pattern '{}'",
        tests.found.len(),
        pattern
    );
    log::info!("command/list_tests done");
    Ok(())
}

/// Show detailed information about tests matching a pattern.
pub fn show_tests(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/show_tests");
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
    log::info!("command/show_tests done");
    Ok(())
}

/// Display detailed information for a single test.
fn show_one_test(test_path: &str, verbosity: u8) -> crate::error::Result<()> {
    let is_rgt = test_path.ends_with(&format!(".{}", crate::rgt::RGT_EXTENSION));
    let name = std::path::Path::new(test_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let tdb_path = if is_rgt {
        crate::rgt::tdb_path_for_rgt(test_path)
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

/// Show details for an .rgt test.
fn show_one_rgt(
    rgt_path: &str,
    name: &str,
    status: &str,
    verbosity: u8,
    tdb_path: &str,
    latest_count: u32,
    diff_count: u32,
) -> crate::error::Result<()> {
    let spec = crate::rgt::parse_rgt(rgt_path)?;

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

    // -v: show baseline output from .out/.err files
    if verbosity >= 1 {
        let baseline_stdout = crate::rgt::read_baseline_stdout(rgt_path)?;
        let baseline_stderr = crate::rgt::read_baseline_stderr(rgt_path)?;
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

    // -vv: show latest results and diffs from .tdb cache
    if verbosity >= 2 && latest_count > 0 {
        show_latest_and_diffs(tdb_path, diff_count)?;
    }

    Ok(())
}

/// Show details for a .tdb test (legacy).
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

    // Show metadata from .tdb
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

    // -v: show baseline output
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

    // -vv: show latest results and diffs
    if verbosity >= 2 && latest_count > 0 {
        show_latest_and_diffs(test_path, diff_count)?;
    }

    Ok(())
}

/// Display latest results and differences (shared by .rgt and .tdb show).
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

/// Read the command for a test, returning (command, tdb_path).
/// Handles both .rgt and .tdb test sources.
fn read_test_command(test_path: &str) -> crate::error::Result<(String, String)> {
    if test_path.ends_with(&format!(".{}", crate::rgt::RGT_EXTENSION)) {
        let spec = crate::rgt::parse_rgt(test_path)?;
        let tdb_path = crate::rgt::tdb_path_for_rgt(test_path);
        Ok((spec.command, tdb_path))
    } else {
        let original = db::read_original_results(test_path)?;
        Ok((original.command, test_path.to_string()))
    }
}

/// Migrate .tdb tests to .rgt text format with .out/.err baselines.
pub fn migrate_tests(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/migrate_tests");
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern.clone())?;
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
        if test_path.ends_with(&format!(".{}", crate::rgt::RGT_EXTENSION)) {
            eprintln!("skip: {} (already .rgt)", test_path);
            continue;
        }
        migrate_one(test_path)?;
        migrated += 1;
    }
    eprintln!("{} test(s) migrated to .rgt format", migrated);
    log::info!("command/migrate_tests done");
    Ok(())
}

/// Migrate a single .tdb file to .rgt + .out + .err format.
fn migrate_one(tdb_path: &str) -> crate::error::Result<()> {
    let original = db::read_original_results(tdb_path)?;
    let rgt_path = std::path::Path::new(tdb_path)
        .with_extension(crate::rgt::RGT_EXTENSION)
        .to_string_lossy()
        .to_string();

    if std::path::Path::new(&rgt_path).exists() {
        eprintln!("skip: {} (.rgt already exists)", tdb_path);
        return Ok(());
    }

    // Read optional metadata from .tdb
    let timeout = db::read_metadata(tdb_path, "timeout")?.and_then(|s| s.parse::<u64>().ok());
    let preprocess = db::read_metadata(tdb_path, crate::preprocess::PREPROCESS_KEY)?;
    let diff_mode = db::read_metadata(tdb_path, crate::normalize::DIFF_MODE_KEY)?;
    let desc = db::read_metadata(tdb_path, META_DESC)?;
    let expects = db::read_metadata(tdb_path, META_EXPECTS)?;
    let flaky_note = db::read_metadata(tdb_path, META_FLAKY_NOTE)?;

    let spec = crate::rgt::RgtSpec {
        command: original.command,
        timeout,
        preprocess,
        diff_mode,
        exit_code: Some(original.exit_code),
        desc,
        expects,
        flaky_note,
    };

    crate::rgt::write_rgt(&rgt_path, &spec)?;
    crate::rgt::write_baseline(&rgt_path, &original.stdout, &original.stderr)?;
    eprintln!("migrated: {} -> {}", tdb_path, rgt_path);
    Ok(())
}

/// Accept latest test output as new baseline, updating .out/.err files.
pub fn rebase_tests(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/rebase_tests");
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern.clone())?;
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
    log::info!("command/rebase_tests done");
    Ok(())
}

/// Rebase a single test: accept latest output as new baseline.
///
/// For .rgt tests: updates .out/.err companion files.
/// For .tdb tests: replaces original results with latest results.
/// Returns true if a rebase was performed.
fn rebase_one(test_path: &str) -> crate::error::Result<bool> {
    let is_rgt = test_path.ends_with(&format!(".{}", crate::rgt::RGT_EXTENSION));
    let tdb_path = if is_rgt {
        crate::rgt::tdb_path_for_rgt(test_path)
    } else {
        test_path.to_string()
    };

    let latest_count = db::count_latest_results(&tdb_path)?;
    if latest_count == 0 {
        eprintln!(
            "skip: {} (no latest results — run the test first)",
            test_path
        );
        return Ok(false);
    }

    let latest = db::read_latest_results(&tdb_path)?;

    if is_rgt {
        // Update .out/.err companion files
        crate::rgt::write_baseline(test_path, &latest.stdout, &latest.stderr)?;
        // Update exit_code in .rgt spec if it has one
        let spec = crate::rgt::parse_rgt(test_path)?;
        if spec.exit_code.is_some() {
            let updated = crate::rgt::RgtSpec {
                exit_code: Some(latest.exit_code),
                ..spec
            };
            crate::rgt::write_rgt(test_path, &updated)?;
        }
    } else {
        // .tdb-only: replace original results with latest
        db::reset_differences(&tdb_path)?;
        db::store_results(
            &tdb_path,
            &runner::TestResults {
                name: latest.name,
                command: latest.command,
                time_created: latest.time_created,
                exit_code: latest.exit_code,
                stdout: latest.stdout,
                stderr: latest.stderr,
            },
            crate::queries::StatementContext::original(),
        )?;
    }

    // Clear diffs since baseline now matches latest
    db::clear_differences(&tdb_path)?;
    eprintln!("rebased: {}", test_path);
    Ok(true)
}

/// Output test names for shell completion.
pub fn complete_tests(config: &config::Config) -> crate::error::Result<()> {
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern)?;
    for test_path in &tests.found {
        let name = std::path::Path::new(test_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        println!("{}", name);
    }
    Ok(())
}

/// Clear latest results from .tdb cache, keeping baselines intact.
pub fn reset_tests(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/reset_tests");
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern.clone())?;
    if tests.found.is_empty() {
        eprintln!(
            "no tests matched pattern '{}' in {}",
            pattern,
            tests.data_dir.display()
        );
        return Ok(());
    }
    let mut reset_count = 0;
    for test_path in &tests.found {
        let is_rgt = test_path.ends_with(&format!(".{}", crate::rgt::RGT_EXTENSION));
        let tdb_path = if is_rgt {
            crate::rgt::tdb_path_for_rgt(test_path)
        } else {
            test_path.to_string()
        };
        db::reset_latest_results(&tdb_path)?;
        db::reset_differences(&tdb_path)?;
        reset_count += 1;
        let name = std::path::Path::new(test_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        eprintln!("reset: {}", name);
    }
    eprintln!("{} test(s) reset", reset_count);
    log::info!("command/reset_tests done");
    Ok(())
}

/// report latest test results
/// Generate reports and return the number of failed tests.
pub fn report_latest(config: &config::Config) -> crate::error::Result<u32> {
    log::info!("command/report_latest");
    if config.is_quiet() {
        // Quiet mode: count failures without printing
        let pattern = config.extract_pattern().to_string();
        let tests = finder::discover(pattern)?;
        let mut fail_count = 0u32;
        for test_path in &tests.found {
            let db = crate::db_path(test_path);
            let latest_count = db::count_latest_results(&db)?;
            if latest_count > 0 && db::count_differences(&db)? > 0 {
                fail_count += 1;
            }
        }
        return Ok(fail_count);
    }
    let fail_count = generate_reports(config)?;
    log::info!("command/report_latest done");
    Ok(fail_count)
}

/// start status client and server
pub async fn status_server(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/status_server");
    status::start_client(config)?;
    status::start_server(config).await?; // loops
    Ok(())
}

/// Metadata keys for self-documenting test information
const META_DESC: &str = "desc";
/// Metadata key for expected behavior
const META_EXPECTS: &str = "expects";
/// Metadata key for flakiness notes
const META_FLAKY_NOTE: &str = "flaky_note";

/// Store documentation metadata (desc, expects, flaky_note) if provided.
fn store_doc_metadata(config: &config::Config, db_name: &str) -> crate::error::Result<()> {
    let (desc, expects, flaky_note) = config.extract_doc_metadata();
    if let Some(d) = desc {
        db::store_metadata(db_name, META_DESC, &d)?;
    }
    if let Some(e) = expects {
        db::store_metadata(db_name, META_EXPECTS, &e)?;
    }
    if let Some(f) = flaky_note {
        db::store_metadata(db_name, META_FLAKY_NOTE, &f)?;
    }
    Ok(())
}

/// Run the --context command and return its stdout, if provided.
fn gather_context(config: &config::Config) -> crate::error::Result<Option<String>> {
    if let Some(context_cmd) = config.extract_context() {
        eprintln!("Running context command: {}", &context_cmd);
        let (_, _, stdout) = crate::process::exec(context_cmd)?;
        Ok(Some(stdout))
    } else {
        Ok(None)
    }
}

/// Gather commands from existing tests in the data directory.
fn gather_existing_test_commands() -> Vec<String> {
    let pattern = String::new(); // match all
    let tests = match finder::discover(pattern) {
        Ok(t) => t.found,
        Err(_) => return vec![],
    };
    let mut commands = Vec::new();
    for test in tests.iter().take(20) {
        if let Ok(result) = db::read_original_results(test) {
            commands.push(result.command.clone());
        }
    }
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_bare_name() {
        let resolved = resolve_test_path("my_test");
        let data_dir = crate::data_dir();
        assert_eq!(resolved, data_dir.join("my_test.rgt").to_string_lossy());
    }

    #[test]
    fn test_resolve_bare_name_with_tdb() {
        // Input has .tdb extension — should convert to .rgt
        let resolved = resolve_test_path("my_test.tdb");
        let data_dir = crate::data_dir();
        assert_eq!(resolved, data_dir.join("my_test.rgt").to_string_lossy());
    }

    #[test]
    fn test_resolve_bare_name_with_rgt() {
        let resolved = resolve_test_path("my_test.rgt");
        let data_dir = crate::data_dir();
        assert_eq!(resolved, data_dir.join("my_test.rgt").to_string_lossy());
    }

    #[test]
    fn test_resolve_path_with_directory() {
        let resolved = resolve_test_path("/tmp/tests/foo");
        assert_eq!(resolved, "/tmp/tests/foo.rgt");
    }

    #[test]
    fn test_resolve_path_with_directory_and_tdb() {
        // Input has .tdb extension — should convert to .rgt
        let resolved = resolve_test_path("/tmp/tests/foo.tdb");
        assert_eq!(resolved, "/tmp/tests/foo.rgt");
    }

    #[test]
    fn test_resolve_relative_path_with_directory() {
        let resolved = resolve_test_path("subdir/foo");
        assert_eq!(resolved, "subdir/foo.rgt");
    }
}

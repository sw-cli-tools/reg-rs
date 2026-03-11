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
    let db_name = resolve_test_path(&test);
    if let Some(test_result) = runner::run_one_timeout(&db_name, &command, false, timeout_secs)? {
        db::reset_differences(&db_name)?;
        db::reset_latest_results(&db_name)?;
        db::store_results(
            &db_name,
            &test_result,
            queries::StatementContext::original(),
        )?;
        if let Some(preprocess) = config.extract_preprocess() {
            db::store_metadata(&db_name, crate::preprocess::PREPROCESS_KEY, &preprocess)?;
        }
        if let Some(diff_mode) = config.extract_diff_mode()
            && diff_mode != "text"
        {
            db::store_metadata(&db_name, crate::normalize::DIFF_MODE_KEY, &diff_mode)?;
        }
        if timeout_secs != 300 {
            db::store_metadata(&db_name, "timeout", &timeout_secs.to_string())?;
        }
        store_doc_metadata(config, &db_name)?;
    }
    Ok(())
}

/// Resolve a test path: if it has no directory component, place it in the
/// data directory. Append `.tdb` extension if missing.
fn resolve_test_path(test: &str) -> String {
    let path = std::path::Path::new(test);
    let mut resolved = if path.parent().is_some_and(|p| p != std::path::Path::new("")) {
        // Has a directory component - use as-is
        path.to_path_buf()
    } else {
        // Just a filename - put it in the data directory
        crate::data_dir().join(test)
    };
    if resolved
        .extension()
        .is_none_or(|ext| ext != crate::TDB_EXTENSION)
    {
        let mut name = resolved.file_name().unwrap_or_default().to_os_string();
        name.push(format!(".{}", crate::TDB_EXTENSION));
        resolved.set_file_name(name);
    }
    resolved.to_string_lossy().to_string()
}

/// Update a test result
pub fn update_latest(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/update_latest");
    runner::run_many(config)?;
    Ok(())
}

/// remove test results
pub fn remove_all(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/remove_all");
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern.clone())?;
    log::debug!("remove_all tests: {:?}", &tests);
    if tests.found.is_empty() {
        eprintln!("warning: no tests matched pattern '{}'", pattern);
        return Ok(());
    }
    for test in &tests.found {
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
    log::info!("command/remove_all done");
    Ok(())
}

/// report latest test results
pub fn report_latest(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/report_latest");
    generate_reports(config)?;
    log::info!("command/report_latest done");
    Ok(())
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
        assert_eq!(resolved, data_dir.join("my_test.tdb").to_string_lossy());
    }

    #[test]
    fn test_resolve_bare_name_with_tdb() {
        let resolved = resolve_test_path("my_test.tdb");
        let data_dir = crate::data_dir();
        assert_eq!(resolved, data_dir.join("my_test.tdb").to_string_lossy());
    }

    #[test]
    fn test_resolve_path_with_directory() {
        let resolved = resolve_test_path("/tmp/tests/foo");
        assert_eq!(resolved, "/tmp/tests/foo.tdb");
    }

    #[test]
    fn test_resolve_path_with_directory_and_tdb() {
        let resolved = resolve_test_path("/tmp/tests/foo.tdb");
        assert_eq!(resolved, "/tmp/tests/foo.tdb");
    }

    #[test]
    fn test_resolve_relative_path_with_directory() {
        let resolved = resolve_test_path("subdir/foo");
        assert_eq!(resolved, "subdir/foo.tdb");
    }
}

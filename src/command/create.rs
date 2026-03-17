use crate::ai;
use crate::config;
use crate::db;
use crate::finder;
use crate::queries;
use crate::rgt;
use crate::runner;

use super::utils::resolve_test_path;

const META_DESC: &str = "desc";
const META_EXPECTS: &str = "expects";
const META_FLAKY_NOTE: &str = "flaky_note";

/// Create a new test
pub fn create(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/create");
    log::debug!("create config: {:?}", &config);

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
    let tdb_path = rgt::tdb_path_for_rgt(&rgt_path);
    if let Some(test_result) = runner::run_one_timeout(&tdb_path, &command, false, timeout_secs)? {
        let preprocess = config.extract_preprocess();
        let diff_mode = config.extract_diff_mode().filter(|m| m != "text");
        let (desc, expects, flaky_note) = config.extract_doc_metadata();
        let spec = rgt::RgtSpec {
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
        rgt::write_rgt(&rgt_path, &spec)?;
        rgt::write_baseline(&rgt_path, &test_result.stdout, &test_result.stderr)?;

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

fn gather_context(config: &config::Config) -> crate::error::Result<Option<String>> {
    if let Some(context_cmd) = config.extract_context() {
        eprintln!("Running context command: {}", &context_cmd);
        let (_, _, stdout) = crate::process::exec(context_cmd)?;
        Ok(Some(stdout))
    } else {
        Ok(None)
    }
}

fn gather_existing_test_commands() -> Vec<String> {
    let pattern = String::new();
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
        let resolved = resolve_test_path("/tmp/tests/foo.tdb");
        assert_eq!(resolved, "/tmp/tests/foo.rgt");
    }

    #[test]
    fn test_resolve_relative_path_with_directory() {
        let resolved = resolve_test_path("subdir/foo");
        assert_eq!(resolved, "subdir/foo.rgt");
    }
}

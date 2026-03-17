use reg_rs_config::config::{Config, CreateOptions};
use reg_rs_runner::runner;
use reg_rs_store::db;
use reg_rs_store::db_ops;
use reg_rs_store::queries;
use reg_rs_store_rgt::rgt;
use reg_rs_types::constants::{DIFF_MODE_KEY, PREPROCESS_KEY};
use reg_rs_types::error::Result;

use crate::utils::resolve_test_path;

const META_DESC: &str = "desc";
const META_EXPECTS: &str = "expects";
const META_FLAKY_NOTE: &str = "flaky_note";

/// Create a new test
pub fn create(config: &Config) -> Result<()> {
    log::info!("command/create");
    log::debug!("create config: {:?}", &config);

    let opts = config.extract_create_options();

    let (test, command) = if let Some(cmd) = opts.command.clone() {
        (opts.test.clone(), cmd)
    } else if let Some(description) = opts.describe.clone() {
        let command = resolve_ai_command(&opts, &description)?;
        match command {
            Some(cmd) => (opts.test.clone(), cmd),
            None => return Ok(()),
        }
    } else {
        return Ok(());
    };

    let rgt_path = resolve_test_path(&test);
    let tdb_path = rgt::tdb_path_for_rgt(&rgt_path);
    if let Some(test_result) = runner::run_one_timeout(&tdb_path, &command, false, opts.timeout)? {
        let diff_mode = if opts.diff_mode == "text" {
            None
        } else {
            Some(opts.diff_mode.clone())
        };
        let spec = build_rgt_spec(&opts, command, &diff_mode, &test_result);
        rgt::write_rgt(&rgt_path, &spec)?;
        rgt::write_baseline(&rgt_path, &test_result.stdout, &test_result.stderr)?;
        store_tdb_state(&opts, &tdb_path, &test_result, &diff_mode)?;
    }
    Ok(())
}

/// Prompt the AI to generate a command, then ask the user to confirm.
/// Returns `None` if the user declines.
fn resolve_ai_command(opts: &CreateOptions, description: &str) -> Result<Option<String>> {
    let context = if let Some(ctx_cmd) = &opts.context {
        reg_rs_ai::context::gather_context(ctx_cmd)?
    } else {
        None
    };
    let existing = reg_rs_ai::context::gather_existing_test_commands();
    let command =
        reg_rs_ai::generate::generate_command(description, context.as_deref(), &existing)?;
    eprintln!("AI generated command: {}", &command);
    eprint!("Proceed? [y/n] ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).map_err(|e| {
        reg_rs_types::error::RegError::Other(format!("Failed to read input: {}", e))
    })?;
    if !input.trim().eq_ignore_ascii_case("y") {
        eprintln!("Aborted.");
        return Ok(None);
    }
    Ok(Some(command))
}

/// Build an RgtSpec from the test result and create options.
fn build_rgt_spec(
    opts: &CreateOptions,
    command: String,
    diff_mode: &Option<String>,
    test_result: &reg_rs_types::types::TestResults,
) -> rgt::RgtSpec {
    rgt::RgtSpec {
        command,
        timeout: if opts.timeout != 300 {
            Some(opts.timeout)
        } else {
            None
        },
        preprocess: opts.preprocess.clone(),
        diff_mode: diff_mode.clone(),
        exit_code: Some(test_result.exit_code),
        desc: opts.desc.clone(),
        expects: opts.expects.clone(),
        flaky_note: opts.flaky_note.clone(),
    }
}

/// Persist test results and metadata into the `.tdb` cache database.
fn store_tdb_state(
    opts: &CreateOptions,
    tdb_path: &str,
    test_result: &reg_rs_types::types::TestResults,
    diff_mode: &Option<String>,
) -> Result<()> {
    db_ops::reset_differences(tdb_path)?;
    db::reset_latest_results(tdb_path)?;
    db::store_results(tdb_path, test_result, queries::StatementContext::original())?;
    if let Some(pp) = &opts.preprocess {
        db_ops::store_metadata(tdb_path, PREPROCESS_KEY, pp)?;
    }
    if let Some(dm) = diff_mode {
        db_ops::store_metadata(tdb_path, DIFF_MODE_KEY, dm)?;
    }
    if opts.timeout != 300 {
        db_ops::store_metadata(tdb_path, "timeout", &opts.timeout.to_string())?;
    }
    store_doc_metadata(opts, tdb_path)?;
    Ok(())
}

/// Store documentation metadata (desc, expects, flaky_note) in the database.
fn store_doc_metadata(opts: &CreateOptions, db_name: &str) -> Result<()> {
    if let Some(d) = &opts.desc {
        db_ops::store_metadata(db_name, META_DESC, d)?;
    }
    if let Some(e) = &opts.expects {
        db_ops::store_metadata(db_name, META_EXPECTS, e)?;
    }
    if let Some(f) = &opts.flaky_note {
        db_ops::store_metadata(db_name, META_FLAKY_NOTE, f)?;
    }
    Ok(())
}

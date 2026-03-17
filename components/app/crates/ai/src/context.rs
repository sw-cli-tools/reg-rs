//! Context gathering for AI command generation.

use reg_rs_store::db;
use reg_rs_types::error::Result;

/// Run a context command and return its stdout.
pub fn gather_context(context_cmd: &str) -> Result<Option<String>> {
    eprintln!("Running context command: {}", context_cmd);
    let (_, _, stdout) = reg_rs_exec::process::exec(context_cmd.to_string())?;
    Ok(Some(stdout))
}

/// Gather existing test commands for AI prompt context.
pub fn gather_existing_test_commands() -> Vec<String> {
    let pattern = String::new();
    let tests = match reg_rs_discover::finder::discover(pattern) {
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

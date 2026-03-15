//! reg-rs (regress) - Regression Test Tool
#![deny(warnings, missing_docs)]
use reg_rs::{analyze, args, builder, command, error::RegError};

/// Exit code when regressions are detected
const EXIT_REGRESSIONS: i32 = 1;

/// Entry point for the application
#[tokio::main]
async fn main() {
    let config = builder::build();
    let default_level = if config.debug { "debug" } else { "warn" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .init();
    log::info!(target: "reg_rs::main", "env_logger initialized");

    let result = run(config).await;

    match result {
        Ok(exit_code) => {
            log::info!(target: "reg_rs::main", "end (exit {})", exit_code);
            std::process::exit(exit_code);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(2);
        }
    }
}

/// Run the selected subcommand and return the exit code.
///
/// Exit codes:
/// - `0`: success (all tests pass, or command completed without error)
/// - `1`: regressions detected (one or more tests failed)
/// - `2`: error (bad args, missing files, etc.) — handled by caller
async fn run(config: reg_rs::config::Config) -> Result<i32, RegError> {
    match &config.mode {
        args::Subcommands::Analyze { pattern } => {
            analyze::analyze_failures(pattern)?;
        }
        args::Subcommands::Complete { .. } => {
            command::complete_tests(&config)?;
        }
        args::Subcommands::Create { .. } => {
            command::create_original(&config)?;
        }
        args::Subcommands::List { .. } => {
            command::list_tests(&config)?;
        }
        args::Subcommands::Migrate { .. } => {
            command::migrate_tests(&config)?;
        }
        args::Subcommands::Rebase { .. } => {
            command::rebase_tests(&config)?;
        }
        args::Subcommands::Reset { .. } => {
            command::reset_tests(&config)?;
        }
        args::Subcommands::Show { .. } => {
            command::show_tests(&config)?;
        }
        args::Subcommands::Remove { .. } => {
            command::remove_all(&config)?;
        }
        args::Subcommands::Report { .. } => {
            let fail_count = command::report_latest(&config)?;
            if fail_count > 0 {
                return Ok(EXIT_REGRESSIONS);
            }
        }
        args::Subcommands::Run { .. } => {
            let fail_count = command::update_latest(&config)?;
            if fail_count > 0 {
                return Ok(EXIT_REGRESSIONS);
            }
        }
        args::Subcommands::Status { .. } => {
            command::status_server(&config).await?;
        }
    }
    Ok(0)
}

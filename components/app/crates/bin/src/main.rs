//! reg-rs (regress) - Regression Test Tool
#![deny(warnings, missing_docs)]

use reg_rs_args::subcommands::Subcommands;
use reg_rs_config::config::Config;
use reg_rs_types::error::RegError;

/// Exit code when regressions are detected
const EXIT_REGRESSIONS: i32 = 1;

/// Entry point for the application
#[tokio::main]
async fn main() {
    let config = reg_rs_config::builder::build();
    let default_level = if config.debug { "debug" } else { "warn" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .init();
    log::info!(target: "reg_rs::main", "env_logger initialized");

    let result = run(config).await;

    match result {
        Ok(exit_code) => {
            log::info!(target: "reg_rs::main", "end (exit {exit_code})");
            std::process::exit(exit_code);
        }
        Err(e) => {
            eprintln!("Error: {e}");
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
async fn run(config: Config) -> Result<i32, RegError> {
    match &config.mode {
        Subcommands::Analyze { pattern } => {
            reg_rs_ai::analyze::analyze_failures(pattern)?;
        }
        Subcommands::Complete { .. } => {
            reg_rs_inspect::complete::complete(&config)?;
        }
        Subcommands::Create { .. } => {
            reg_rs_commands::create::create(&config)?;
        }
        Subcommands::List { .. } => {
            reg_rs_inspect::list::list(&config)?;
        }
        Subcommands::Migrate { .. } => {
            reg_rs_modify::migrate::migrate(&config)?;
        }
        Subcommands::Rebase { .. } => {
            reg_rs_modify::rebase::rebase(&config)?;
        }
        Subcommands::Reset { .. } => {
            reg_rs_modify::reset::reset(&config)?;
        }
        Subcommands::Show { .. } => {
            reg_rs_inspect::show::show(&config)?;
        }
        Subcommands::Remove { .. } => {
            reg_rs_modify::remove::remove(&config)?;
        }
        Subcommands::Report { .. } => {
            let fail_count = reg_rs_commands::report::report(&config)?;
            if fail_count > 0 {
                return Ok(EXIT_REGRESSIONS);
            }
        }
        Subcommands::Run { .. } => {
            let fail_count = reg_rs_commands::run::run(&config)?;
            if fail_count > 0 {
                return Ok(EXIT_REGRESSIONS);
            }
        }
        Subcommands::Status { .. } => {
            reg_rs_modify::status::status(&config).await?;
        }
    }
    Ok(0)
}

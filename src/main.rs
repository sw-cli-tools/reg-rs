//! reg-rs (regress) - Regression Test Tool
#![deny(warnings, missing_docs)]
use reg_rs::{analyze, args, builder, command, error::RegError};

/// Entry point for the application
#[tokio::main]
async fn main() -> Result<(), RegError> {
    let config = builder::build();
    let default_level = if config.debug { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .init();
    log::info!(target: "reg_rs::main", "env_logger initialized");
    match &config.mode {
        args::Subcommands::Analyze { pattern } => {
            analyze::analyze_failures(pattern)?;
        }
        args::Subcommands::Create { .. } => {
            command::create_original(&config)?;
        }
        args::Subcommands::List { .. } => {
            command::list_tests(&config)?;
        }
        args::Subcommands::Remove { .. } => {
            command::remove_all(&config)?;
        }
        args::Subcommands::Report { .. } => {
            command::report_latest(&config)?;
        }
        args::Subcommands::Run { .. } => {
            command::update_latest(&config)?;
        }
        args::Subcommands::Status { .. } => {
            command::status_server(&config).await?;
        }
    }
    log::info!(target: "reg_rs::main", "end");
    Ok(())
}

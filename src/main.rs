//! Regression Test Tool
#![deny(warnings, missing_docs)]
use rtt1::{args, builder, command, error::RttError};

/// Entry point for the application
#[tokio::main]
async fn main() -> Result<(), RttError> {
    env_logger::init();
    log::info!(target: "rtt1::main", "env_logger initialized");
    let config = builder::build();
    match &config.mode {
        args::Subcommands::Create { .. } => {
            command::create_original(&config)?;
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
    log::info!(target: "rtt1::main", "end");
    Ok(())
}

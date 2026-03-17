use std::env;

use clap::Parser;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub use crate::subcommands::Subcommands;

/// Argument processing configuration
#[derive(Debug, PartialEq, Parser)]
#[clap(
    name = "reg-rs",
    version = generated_version(),
    long_version = extended_version(),
    about = "reg-rs (regress) - A CLI for regression testing",
    long_about = generated_long_about()
)]
pub struct Args {
    /// Enable debug logging
    #[clap(long, short)]
    pub debug: bool,

    /// Enable file logging
    #[clap(long, short)]
    pub logging: bool,

    /// Subcommand to execute
    #[clap(subcommand)]
    pub command: Subcommands,
}

/// Parse command-line arguments
pub fn parse_args() -> Args {
    Args::parse()
}

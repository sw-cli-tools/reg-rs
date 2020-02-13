use rtt1::args;
use rtt1::builder;
use rtt1::db;
use rtt1::report;
use rtt1::runner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = builder::build();
    let tests = runner::discover(&config)?;
    match &config.mode {
        args::Subcommands::Create { test: _, command: _ } => {
            db::create(&config)?;
        }
        args::Subcommands::Report { pattern: _ } => {
            report::generate(&config)?;
        },
        args::Subcommands::Run{dry_run: _, pattern: _ } => {
            runner::run_many(&config, &tests)?;
        },
    }
    Ok(())
}

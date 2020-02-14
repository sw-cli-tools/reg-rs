use rtt1::args;
use rtt1::builder;
use rtt1::db;
use rtt1::report;
use rtt1::runner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = builder::build();
    match &config.mode {
        args::Subcommands::Create { .. } => {
            db::create(&config)?;
        }
        args::Subcommands::Report { .. } => {
            report::generate(&config)?;
        }
        args::Subcommands::Run { .. } => {
            runner::run_many(&config)?;
        }
    }
    Ok(())
}

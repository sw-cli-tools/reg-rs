use rtt1::args;
use rtt1::builder;
use rtt1::command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    }
    Ok(())
}

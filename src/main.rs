use rtt1::{args, builder, command};

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
        args::Subcommands::Status { .. } => {
            command::status_server(&config)?;
        }
    }
    Ok(())
}

use log;
use crate::config;

pub mod client;
pub mod monitor;
pub mod server;
pub mod views;

pub fn start_client(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("status/start_client");
    let status_port = config.status_port();
    client::start(status_port);
    Ok(())
}

pub fn start_server(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("status/start_server");
    server::start(&config)?;
    Ok(())
}

use crate::config;

pub mod client;
pub mod server;
pub mod views;

pub fn start_client(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let status_port = config.status_port();
    client::start(status_port);
    Ok(())
}

pub fn start_server(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    server::start(&config)?;
    Ok(())
}

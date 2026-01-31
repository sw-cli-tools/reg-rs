use crate::config;

/// client
pub mod client;
/// monitor
pub mod monitor;
/// server
pub mod server;
/// views
pub mod views;

/// start client
pub fn start_client(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("status/start_client");
    let status_port = config.status_port();
    client::start(status_port);
    Ok(())
}

/// start server
pub async fn start_server(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("status/start_server");
    server::start(config).await?;
    Ok(())
}

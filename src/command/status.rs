use crate::config;
use crate::status;

/// Start status monitoring server
pub async fn status(config: &config::Config) -> crate::error::Result<()> {
    log::info!("command/status");
    status::start_client(config.status_port());
    status::start_server(config).await?;
    Ok(())
}

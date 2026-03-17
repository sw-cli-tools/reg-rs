use reg_rs_config::config::Config;
use reg_rs_types::error::Result;

/// Start status monitoring server
pub async fn status(config: &Config) -> Result<()> {
    log::info!("command/status");
    let port = config.status_port();
    let pattern = config.extract_pattern().to_string();
    let data_dir = reg_rs_discover::data_dir::data_dir();

    println!("open: http://localhost:{}/", port);

    // Discover tests upfront so the server has them on first request
    let initial_tests = reg_rs_discover::finder::discover(pattern.clone())
        .map(|r| r.found)
        .unwrap_or_default();

    reg_rs_status::server::start(&pattern, port, data_dir, &initial_tests).await?;
    Ok(())
}

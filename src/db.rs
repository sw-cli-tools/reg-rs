use crate::config;

pub fn create(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    if config.debug {
        dbg!(&config);
    }
    Ok(())
}

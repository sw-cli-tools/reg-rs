use crate::config;
use crate::db;
use crate::runner;

pub fn generate(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let tests = runner::discover(&config)?;
    if config.debug {
        md!(&config);
    }
    let test_result = db::open_read("data/rtt01.tdb")?; // TODO loop through discovered tests
    md!(test_result);
    Ok(())
}

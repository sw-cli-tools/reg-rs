use std::process;

use crate::time;

/// Set up file logging with the given level filter
pub fn setup_logging(level: log::LevelFilter) -> reg_rs_types::error::Result<()> {
    let pid = process::id();
    let file_name = format!("reg-rs-{pid}.log");
    eprintln!("logging/setup_logging file: {}", &file_name);
    fern::Dispatch::new()
        .level(level)
        .chain(fern::DateBased::new(&file_name, "%Y-%m-%d"))
        .apply()
        .map_err(|e| reg_rs_types::error::RegError::Other(e.to_string()))?;
    log::info!("setup_logging {}", time::now());
    Ok(())
}

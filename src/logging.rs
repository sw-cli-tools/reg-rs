use std::process;

use crate::time;

/// Set up logging
pub(crate) fn setup_logging(level: log::LevelFilter) -> crate::error::Result<()> {
    let pid = process::id();
    let file_name = format!("reg-rs-{}.log", pid);
    eprintln!("logging/setup_logging file: {}", &file_name);
    fern::Dispatch::new()
        .level(level)
        .chain(fern::DateBased::new(&file_name, "%Y-%m-%d"))
        .apply()
        .map_err(|e| crate::error::RegError::Other(e.to_string()))?;
    log::info!("setup_logging {}", time::now());
    Ok(())
}

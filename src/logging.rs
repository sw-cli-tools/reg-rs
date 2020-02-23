use log;
use std::process;


use crate::time;

pub fn setup_logging(level: log::LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    let pid = process::id();
    let file_name = format!("rtt1-{}.log", pid);
    println!("logging/setup_logging file: {}", &file_name);
    fern::Dispatch::new()
        .level(level)
        .chain(fern::DateBased::new(&file_name, "%Y-%m-%d"))
        .apply()?;
    log::info!("setup_logging {}", time::now());
    Ok(())
}

use reg_rs_args::args;

use crate::config;
use crate::logging;

/// Build configuration based on arguments
pub fn build() -> config::Config {
    let args = args::parse_args();
    if args.logging {
        logging::setup_logging(log::LevelFilter::Debug).expect("failed to initialize file logging");
    }
    let config = config::Config {
        mode: args.command,
        debug: args.debug,
    };
    log::info!("config {:?}", &config);
    config
}

use crate::args;
use crate::config;
pub fn build() -> config::Config {
    let args = args::parse_args();
    let config = config::Config {
        mode: args.command,
        debug: args.debug,
    };
    if config.debug {
        md!(&config);
    }
    config
}

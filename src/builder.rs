use crate::args;
use crate::config;
pub fn build() -> config::Config {
    let args = args::parse_args();
    config::Config {
        mode: args.command,
        debug: args.debug,
    }
}

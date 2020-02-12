use crate::args;
use crate::config;
pub fn build() -> config::Config {
    config::Config { mode: args::subcommands() }
}

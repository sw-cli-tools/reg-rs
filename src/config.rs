use crate::args;
#[derive(Debug)]
pub struct Config {
    pub mode: args::Subcommands,
}

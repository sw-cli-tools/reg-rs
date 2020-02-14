use crate::args;

#[derive(Debug)]
pub struct Config {
    pub mode: args::Subcommands,
    pub debug: bool,
}

impl Config {
    pub fn extract_test_and_command(self: &Self) -> Option<(String, String)> {
        if let args::Subcommands::Create { test, command } = &self.mode {
            md!((&test, &command));
            Some((test.to_string(), command.to_string()))
        } else {
            None
        }
    }
}

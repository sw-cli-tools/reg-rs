use crate::args;

#[derive(Debug)]
pub struct Config {
    pub mode: args::Subcommands,
    pub debug: bool,
}

// TODO impl on Config with error result
pub fn extract_test_and_command(config: &Config) -> Option<(String, String)> {
    if let args::Subcommands::Create { test, command } = &config.mode {
        dbg!((&test, &command));
        Some((test.to_string(), command.to_string()))
    } else {
        None
    }
}

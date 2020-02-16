use crate::args;

#[derive(Debug)]
pub struct Config {
    pub mode: args::Subcommands,
    pub debug: bool,
}

impl Config {
    pub fn is_dry_run(self: &Self) -> bool {
        match &self.mode {
            args::Subcommands::Run { dry_run, .. } => *dry_run,
            _ => false,
        }
    }
    
    pub fn extract_pattern(self: &Self) -> &str {
        let default_pattern = ".tdb";
        match &self.mode {
            args::Subcommands::Report { pattern } => pattern,
            args::Subcommands::Run { pattern, .. } => pattern,
            _ => default_pattern,
        }
    }

    pub fn extract_test_and_command(self: &Self) -> Option<(String, String)> {
        if let args::Subcommands::Create { test, command } = &self.mode {
            md!((&test, &command));
            Some((test.to_string(), command.to_string()))
        } else {
            None
        }
    }
}


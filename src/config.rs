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
        md!(&self.mode);
        let p = match &self.mode {
            args::Subcommands::Create { .. } => unreachable!(),
            args::Subcommands::Remove { pattern, .. } => pattern,
            args::Subcommands::Report { pattern, .. } => pattern,
            args::Subcommands::Run { pattern, .. } => pattern,
            args::Subcommands::Status { pattern, .. } => pattern,
        };
        md!(&p);
        p
    }

    pub fn extract_test_and_command(self: &Self) -> Option<(String, String)> {
        if let args::Subcommands::Create { test, command } = &self.mode {
            md!((&test, &command));
            Some((test.to_string(), command.to_string()))
        } else {
            None
        }
    }

    pub fn verbosity_level(self: &Self) -> u8 {
        if let args::Subcommands::Report { verbosity, .. } = &self.mode {
            md!(verbosity);
            *verbosity
        } else {
            0
        }
    }

    pub fn status_port(self: &Self) -> u16 {
        if let args::Subcommands::Status { localhost_port, .. } = &self.mode {
            *localhost_port
        } else {
            crate::DEFAULT_STATUS_PORT
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO test is_dry_run
    // TODO test extract_test_and_command
    
    #[test]
    fn test_extract_report_pattern() {
        let args = args::Args {
            command: args::Subcommands::Report {
                pattern: "foo".to_string(),
                verbosity: 0
            },
            debug: false,
        };
        assert_eq!("foo".to_string(),
                   Config {
                       mode: args.command,
                       debug: false,
                   }.extract_pattern());
    }

    #[test]
    fn test_extract_run_pattern() {
        let args = args::Args {
            command: args::Subcommands::Run {
                dry_run: false,
                pattern: "bar".to_string(),
            },
            debug: false,
        };
        assert_eq!("bar".to_string(),
                   Config {
                       mode: args.command,
                       debug: false,
                   }.extract_pattern());
    }

    #[test]
    fn test_default_verbosity_level() {
        let args = args::Args {
            command: args::Subcommands::Report {
                pattern: "foo".to_string(),
                verbosity: 0
            },
            debug: false,
        };
        assert_eq!(0,
                   Config {
                       mode: args.command,
                       debug: false,
                   }.verbosity_level());
    }

    #[test]
    fn test_non_default_verbosity_level() {
        let args = args::Args {
            command: args::Subcommands::Report {
                pattern: "foo".to_string(),
                verbosity: 3
            },
            debug: false,
        };
        assert_eq!(3,
                   Config {
                       mode: args.command,
                       debug: false,
                   }.verbosity_level());
    }
    
}

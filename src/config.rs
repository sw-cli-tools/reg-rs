use crate::args;

/// Configuration data
#[derive(Debug)]
pub struct Config {
    /// subcommands
    pub mode: args::Subcommands,
    /// debugging flag
    pub debug: bool,
}

impl Config {
    /// determines if dry-run flag was specified
    pub fn is_dry_run(&self) -> bool {
        match &self.mode {
            args::Subcommands::Run { dry_run, .. } => *dry_run,
            _ => false,
        }
    }

    /// extracts the test name pattern
    pub fn extract_pattern(&self) -> &str {
        log::debug!("extract_pattern mode: {:?}", &self.mode);
        let p = match &self.mode {
            args::Subcommands::Create { .. } => unreachable!(),
            args::Subcommands::Remove { pattern, .. } => pattern,
            args::Subcommands::Report { pattern, .. } => pattern,
            args::Subcommands::Run { pattern, .. } => pattern,
            args::Subcommands::Status { pattern, .. } => pattern,
        };
        log::debug!("extract_pattern result: {:?}", &p);
        p
    }

    /// extract test and command
    pub fn extract_test_and_command(&self) -> Option<(String, String)> {
        if let args::Subcommands::Create { test, command } = &self.mode {
            log::debug!("extract_test_and_command: {:?}, {:?}", &test, &command);
            Some((test.to_string(), command.to_string()))
        } else {
            None
        }
    }

    /// determines how verbose the output should be
    pub fn verbosity_level(&self) -> u8 {
        if let args::Subcommands::Report { verbosity, .. } = &self.mode {
            log::debug!("verbosity_level: {}", verbosity);
            *verbosity
        } else {
            0
        }
    }

    /// determines which port to use for web server
    pub fn status_port(&self) -> u16 {
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
                verbosity: 0,
            },
            debug: false,
            logging: false,
        };
        assert_eq!(
            "foo".to_string(),
            Config {
                mode: args.command,
                debug: false,
            }
            .extract_pattern()
        );
    }

    #[test]
    fn test_extract_run_pattern() {
        let args = args::Args {
            command: args::Subcommands::Run {
                dry_run: false,
                pattern: "bar".to_string(),
            },
            debug: false,
            logging: false,
        };
        assert_eq!(
            "bar".to_string(),
            Config {
                mode: args.command,
                debug: false,
            }
            .extract_pattern()
        );
    }

    #[test]
    fn test_default_verbosity_level() {
        let args = args::Args {
            command: args::Subcommands::Report {
                pattern: "foo".to_string(),
                verbosity: 0,
            },
            debug: false,
            logging: false,
        };
        assert_eq!(
            0,
            Config {
                mode: args.command,
                debug: false,
            }
            .verbosity_level()
        );
    }

    #[test]
    fn test_non_default_verbosity_level() {
        let args = args::Args {
            command: args::Subcommands::Report {
                pattern: "foo".to_string(),
                verbosity: 3,
            },
            debug: false,
            logging: false,
        };
        assert_eq!(
            3,
            Config {
                mode: args.command,
                debug: false,
            }
            .verbosity_level()
        );
    }
}

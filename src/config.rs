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

    /// extract test and command (command is present when using -c)
    pub fn extract_test_and_command(&self) -> Option<(String, String)> {
        if let args::Subcommands::Create {
            test,
            command: Some(command),
            ..
        } = &self.mode
        {
            log::debug!("extract_test_and_command: {:?}, {:?}", &test, &command);
            Some((test.to_string(), command.to_string()))
        } else {
            None
        }
    }

    /// extract test name and description (description is present when using -D)
    pub fn extract_test_and_describe(&self) -> Option<(String, String)> {
        if let args::Subcommands::Create {
            test,
            describe: Some(describe),
            ..
        } = &self.mode
        {
            log::debug!("extract_test_and_describe: {:?}, {:?}", &test, &describe);
            Some((test.to_string(), describe.to_string()))
        } else {
            None
        }
    }

    /// extract optional preprocess command from Create subcommand
    pub fn extract_preprocess(&self) -> Option<String> {
        if let args::Subcommands::Create {
            preprocess: Some(preprocess),
            ..
        } = &self.mode
        {
            Some(preprocess.to_string())
        } else {
            None
        }
    }

    /// extract diff mode from Create subcommand
    pub fn extract_diff_mode(&self) -> Option<String> {
        if let args::Subcommands::Create { diff_mode, .. } = &self.mode {
            Some(diff_mode.clone())
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

    #[test]
    fn test_is_dry_run_true() {
        let config = Config {
            mode: args::Subcommands::Run {
                dry_run: true,
                pattern: "foo".to_string(),
            },
            debug: false,
        };
        assert!(config.is_dry_run());
    }

    #[test]
    fn test_is_dry_run_false() {
        let config = Config {
            mode: args::Subcommands::Run {
                dry_run: false,
                pattern: "foo".to_string(),
            },
            debug: false,
        };
        assert!(!config.is_dry_run());
    }

    #[test]
    fn test_is_dry_run_non_run_command() {
        let config = Config {
            mode: args::Subcommands::Report {
                pattern: "foo".to_string(),
                verbosity: 0,
            },
            debug: false,
        };
        assert!(!config.is_dry_run());
    }

    #[test]
    fn test_extract_test_and_command() {
        let config = Config {
            mode: args::Subcommands::Create {
                test: "my_test".to_string(),
                command: Some("echo hello".to_string()),
                describe: None,
                preprocess: None,
                diff_mode: "text".to_string(),
            },
            debug: false,
        };
        let result = config.extract_test_and_command();
        assert_eq!(
            result,
            Some(("my_test".to_string(), "echo hello".to_string()))
        );
    }

    #[test]
    fn test_extract_test_and_command_non_create() {
        let config = Config {
            mode: args::Subcommands::Run {
                dry_run: false,
                pattern: "foo".to_string(),
            },
            debug: false,
        };
        assert_eq!(config.extract_test_and_command(), None);
    }

    #[test]
    fn test_status_port_default() {
        let config = Config {
            mode: args::Subcommands::Report {
                pattern: "foo".to_string(),
                verbosity: 0,
            },
            debug: false,
        };
        assert_eq!(config.status_port(), crate::DEFAULT_STATUS_PORT);
    }

    #[test]
    fn test_status_port_custom() {
        let config = Config {
            mode: args::Subcommands::Status {
                pattern: "foo".to_string(),
                localhost_port: 8080,
            },
            debug: false,
        };
        assert_eq!(config.status_port(), 8080);
    }

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

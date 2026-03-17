use reg_rs_args::args::Args;
use reg_rs_args::subcommands::Subcommands;
use reg_rs_config::config::Config;

#[test]
fn test_extract_report_pattern() {
    let args = Args {
        command: Subcommands::Report {
            pattern: "foo".to_string(),
            verbosity: 0,
            quiet: false,
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
    let args = Args {
        command: Subcommands::Run {
            dry_run: false,
            pattern: "bar".to_string(),
            parallel: false,
            verbosity: 0,
            quiet: false,
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
    let args = Args {
        command: Subcommands::Report {
            pattern: "foo".to_string(),
            verbosity: 0,
            quiet: false,
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
    let args = Args {
        command: Subcommands::Report {
            pattern: "foo".to_string(),
            verbosity: 3,
            quiet: false,
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

#[test]
#[should_panic(expected = "extract_create_options called on non-Create config")]
fn test_extract_create_options_non_create() {
    let config = Config {
        mode: Subcommands::Run {
            dry_run: false,
            pattern: "foo".to_string(),
            parallel: false,
            verbosity: 0,
            quiet: false,
        },
        debug: false,
    };
    let _ = config.extract_create_options();
}

use reg_rs_args::subcommands::Subcommands;
use reg_rs_config::config::Config;

#[test]
fn test_is_dry_run_true() {
    let config = Config {
        mode: Subcommands::Run {
            dry_run: true,
            pattern: "foo".to_string(),
            parallel: false,
            verbosity: 0,
            quiet: false,
        },
        debug: false,
    };
    assert!(config.is_dry_run());
}

#[test]
fn test_is_dry_run_false() {
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
    assert!(!config.is_dry_run());
}

#[test]
fn test_is_dry_run_non_run_command() {
    let config = Config {
        mode: Subcommands::Report {
            pattern: "foo".to_string(),
            verbosity: 0,
            quiet: false,
        },
        debug: false,
    };
    assert!(!config.is_dry_run());
}

#[test]
fn test_extract_create_options() {
    let config = Config {
        mode: Subcommands::Create {
            test: "my_test".to_string(),
            command: Some("echo hello".to_string()),
            describe: None,
            context: None,
            preprocess: None,
            diff_mode: "text".to_string(),
            timeout: 300,
            desc: None,
            expects: None,
            flaky_note: None,
        },
        debug: false,
    };
    let opts = config.extract_create_options();
    assert_eq!(opts.test, "my_test");
    assert_eq!(opts.command, Some("echo hello".to_string()));
    assert_eq!(opts.diff_mode, "text");
    assert_eq!(opts.timeout, 300);
}

#[test]
fn test_status_port_default() {
    let config = Config {
        mode: Subcommands::Report {
            pattern: "foo".to_string(),
            verbosity: 0,
            quiet: false,
        },
        debug: false,
    };
    assert_eq!(
        config.status_port(),
        reg_rs_types::constants::DEFAULT_STATUS_PORT
    );
}

#[test]
fn test_status_port_custom() {
    let config = Config {
        mode: Subcommands::Status {
            pattern: "foo".to_string(),
            localhost_port: 8080,
        },
        debug: false,
    };
    assert_eq!(config.status_port(), 8080);
}

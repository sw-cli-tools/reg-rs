#[test]
fn test_create_defaults() {
    assert_eq!(
        super::Args {
            command: super::Subcommands::Create {
                test: "pat001".to_string(),
                command: Some("pwd".to_string()),
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
            logging: false,
        },
        <super::Args as clap::Parser>::try_parse_from([
            "test", "create", "-t", "pat001", "-c", "pwd"
        ])
        .unwrap()
    );
}

#[test]
fn test_create_no_defaults() {
    assert_eq!(
        super::Args {
            command: super::Subcommands::Create {
                test: "pat001".to_string(),
                command: Some("pwd".to_string()),
                describe: None,
                context: None,
                preprocess: None,
                diff_mode: "text".to_string(),
                timeout: 300,
                desc: None,
                expects: None,
                flaky_note: None,
            },
            debug: true,
            logging: false,
        },
        <super::Args as clap::Parser>::try_parse_from([
            "test", "-d", "create", "-t", "pat001", "-c", "pwd"
        ])
        .unwrap()
    );
}

#[test]
fn test_remove_defaults() {
    assert_eq!(
        super::Args {
            command: super::Subcommands::Remove {
                pattern: "pat001".to_string(),
            },
            debug: false,
            logging: false,
        },
        <super::Args as clap::Parser>::try_parse_from(["test", "remove", "-p", "pat001"]).unwrap()
    );
}

#[test]
fn test_remove_no_defaults() {
    assert_eq!(
        super::Args {
            command: super::Subcommands::Remove {
                pattern: "pat001".to_string(),
            },
            debug: true,
            logging: false,
        },
        <super::Args as clap::Parser>::try_parse_from(["test", "-d", "remove", "-p", "pat001"])
            .unwrap()
    );
}

#[test]
fn test_report_defaults() {
    assert_eq!(
        super::Args {
            command: super::Subcommands::Report {
                pattern: "pat001".to_string(),
                verbosity: 0,
                quiet: false,
            },
            debug: false,
            logging: false,
        },
        <super::Args as clap::Parser>::try_parse_from(["test", "report", "-p", "pat001"]).unwrap()
    );
}

#[test]
fn test_report_no_defaults() {
    assert_eq!(
        super::Args {
            command: super::Subcommands::Report {
                pattern: "pat001".to_string(),
                verbosity: 3,
                quiet: false,
            },
            debug: true,
            logging: false,
        },
        <super::Args as clap::Parser>::try_parse_from([
            "test", "-d", "report", "-p", "pat001", "-vvv"
        ])
        .unwrap()
    );
}

#[test]
fn test_run_defaults() {
    assert_eq!(
        super::Args {
            command: super::Subcommands::Run {
                dry_run: false,
                pattern: "pat001".to_string(),
                parallel: false,
                verbosity: 0,
                quiet: false,
            },
            debug: false,
            logging: false,
        },
        <super::Args as clap::Parser>::try_parse_from(["test", "run", "-p", "pat001"]).unwrap()
    );
}

#[test]
fn test_show_defaults() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "show"]).unwrap();
    match args.command {
        super::Subcommands::Show { pattern, verbosity } => {
            assert_eq!(pattern, ".tdb");
            assert_eq!(verbosity, 0);
        }
        _ => panic!("expected Show"),
    }
}

#[test]
fn test_show_with_verbosity() {
    let args =
        <super::Args as clap::Parser>::try_parse_from(["test", "show", "-p", "hello", "-vv"])
            .unwrap();
    match args.command {
        super::Subcommands::Show { pattern, verbosity } => {
            assert_eq!(pattern, "hello");
            assert_eq!(verbosity, 2);
        }
        _ => panic!("expected Show"),
    }
}

#[test]
fn test_show_alias() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "w", "-p", "foo"]).unwrap();
    match args.command {
        super::Subcommands::Show { pattern, .. } => assert_eq!(pattern, "foo"),
        _ => panic!("expected Show"),
    }
}

#[test]
fn test_list_defaults() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "list"]).unwrap();
    match args.command {
        super::Subcommands::List { pattern } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected List"),
    }
}

#[test]
fn test_list_with_pattern() {
    let args =
        <super::Args as clap::Parser>::try_parse_from(["test", "list", "-p", "hello"]).unwrap();
    match args.command {
        super::Subcommands::List { pattern } => assert_eq!(pattern, "hello"),
        _ => panic!("expected List"),
    }
}

#[test]
fn test_list_alias() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "l", "-p", "foo"]).unwrap();
    match args.command {
        super::Subcommands::List { pattern } => assert_eq!(pattern, "foo"),
        _ => panic!("expected List"),
    }
}

#[test]
fn test_migrate_defaults() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "migrate"]).unwrap();
    match args.command {
        super::Subcommands::Migrate { pattern } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected Migrate"),
    }
}

#[test]
fn test_migrate_with_pattern() {
    let args =
        <super::Args as clap::Parser>::try_parse_from(["test", "migrate", "-p", "hello"]).unwrap();
    match args.command {
        super::Subcommands::Migrate { pattern } => assert_eq!(pattern, "hello"),
        _ => panic!("expected Migrate"),
    }
}

#[test]
fn test_migrate_alias() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "m", "-p", "foo"]).unwrap();
    match args.command {
        super::Subcommands::Migrate { pattern } => assert_eq!(pattern, "foo"),
        _ => panic!("expected Migrate"),
    }
}

#[test]
fn test_rebase_defaults() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "rebase"]).unwrap();
    match args.command {
        super::Subcommands::Rebase { pattern } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected Rebase"),
    }
}

#[test]
fn test_rebase_with_pattern() {
    let args =
        <super::Args as clap::Parser>::try_parse_from(["test", "rebase", "-p", "hello"]).unwrap();
    match args.command {
        super::Subcommands::Rebase { pattern } => assert_eq!(pattern, "hello"),
        _ => panic!("expected Rebase"),
    }
}

#[test]
fn test_rebase_alias() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "u", "-p", "foo"]).unwrap();
    match args.command {
        super::Subcommands::Rebase { pattern } => assert_eq!(pattern, "foo"),
        _ => panic!("expected Rebase"),
    }
}

#[test]
fn test_reset_defaults() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "reset"]).unwrap();
    match args.command {
        super::Subcommands::Reset { pattern } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected Reset"),
    }
}

#[test]
fn test_reset_with_pattern() {
    let args =
        <super::Args as clap::Parser>::try_parse_from(["test", "reset", "-p", "hello"]).unwrap();
    match args.command {
        super::Subcommands::Reset { pattern } => assert_eq!(pattern, "hello"),
        _ => panic!("expected Reset"),
    }
}

#[test]
fn test_complete_defaults() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "complete"]).unwrap();
    match args.command {
        super::Subcommands::Complete { pattern } => assert_eq!(pattern, ".rgt"),
        _ => panic!("expected Complete"),
    }
}

#[test]
fn test_complete_with_pattern() {
    let args =
        <super::Args as clap::Parser>::try_parse_from(["test", "complete", "-p", "hello"]).unwrap();
    match args.command {
        super::Subcommands::Complete { pattern } => assert_eq!(pattern, "hello"),
        _ => panic!("expected Complete"),
    }
}

#[test]
fn test_run_default_pattern() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "run"]).unwrap();
    match args.command {
        super::Subcommands::Run { pattern, .. } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected Run"),
    }
}

#[test]
fn test_report_default_pattern() {
    let args = <super::Args as clap::Parser>::try_parse_from(["test", "report"]).unwrap();
    match args.command {
        super::Subcommands::Report { pattern, .. } => assert_eq!(pattern, ".tdb"),
        _ => panic!("expected Report"),
    }
}

#[test]
fn test_run_no_defaults() {
    assert_eq!(
        super::Args {
            command: super::Subcommands::Run {
                dry_run: true,
                pattern: "pat001".to_string(),
                parallel: false,
                verbosity: 0,
                quiet: false,
            },
            debug: true,
            logging: false,
        },
        <super::Args as clap::Parser>::try_parse_from(["test", "-d", "run", "-p", "pat001", "-n"])
            .unwrap()
    );
}

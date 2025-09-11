use std::process::Command;

mod common;

#[test]
fn integration_test_rtt1_help() {
    common::setup();
    let command = "./target/debug/rtt1 -h";
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
    let status_code = match output.status.code() {
        Some(n) => n,
        _ => 0,
    };
    assert_eq!(0, status_code);
    assert_eq!("", String::from_utf8_lossy(&output.stderr));
    assert_eq!(
        "Argument processing configuration

Usage: rtt1 [OPTIONS] <COMMAND>

Commands:
  create  Creates a new test of a specified command (alias c)
  remove  Removes previously created test and run results if any.  Discards test and results!
  report  Reports counts/summary of specified test(s) (alias p)
  run     Runs a test (or tests) based on a test name pattern (alias r)
  status  Starts a server to monitor long running tests and/or show results (alias s)
  help    Print this message or the help of the given subcommand(s)

Options:
  -d, --debug    Prints debugging info.  -d must preceed subcommands
  -l, --logging  Logs to a log file.  -l must preceed subcommands
  -h, --help     Print help
  -V, --version  Print version
",
        String::from_utf8_lossy(&output.stdout).to_string()
    );
}

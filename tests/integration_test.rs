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
    assert_eq!("rtt1 0.1.0\nRegression Test Tool (first draft) - create, report, and run tests\nfor more details:\n rtt1 create --help\n rtt1 run --help\n rtt1 remove --help\n rtt1 report --help\n\nUSAGE:\n    rtt1 [FLAGS] <SUBCOMMAND>\n\nFLAGS:\n    -d, --debug      Prints debugging info.  -d must preceed subcommands\n    -h, --help       Prints help information\n    -V, --version    Prints version information\n\nSUBCOMMANDS:\n    create    Creates a new test of a specified command\n    help      Prints this message or the help of the given subcommand(s)\n    remove    Removes previously created test and run results if any.  Discards test and results!\n    report    Reports counts/summary of specified test(s)\n    run       Runs a test (or tests) based on a test name pattern\n".to_string(),
        String::from_utf8_lossy(&output.stdout).to_string());
}

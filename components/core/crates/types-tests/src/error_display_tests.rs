use reg_rs_types::error::RegError;
use std::path::PathBuf;

#[test]
fn test_display_io_error() {
    let err = RegError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
    assert!(err.to_string().contains("missing"));
}

#[test]
fn test_display_file_lock() {
    let err = RegError::FileLock("locked".to_string());
    assert_eq!(err.to_string(), "File lock error: locked");
}

#[test]
fn test_display_test_not_found() {
    let err = RegError::TestNotFound("my_test".to_string());
    assert_eq!(err.to_string(), "Test not found: my_test");
}

#[test]
fn test_display_command_execution() {
    let err = RegError::CommandExecution("timeout".to_string());
    assert_eq!(err.to_string(), "Command execution failed: timeout");
}

#[test]
fn test_display_path_error() {
    let err = RegError::Path {
        path: PathBuf::from("/tmp/test"),
        message: "not found".to_string(),
    };
    assert!(err.to_string().contains("/tmp/test"));
    assert!(err.to_string().contains("not found"));
}

#[test]
fn test_display_template() {
    let err = RegError::Template("bad template".to_string());
    assert_eq!(err.to_string(), "Template error: bad template");
}

#[test]
fn test_display_other() {
    let err = RegError::Other("something".to_string());
    assert_eq!(err.to_string(), "something");
}

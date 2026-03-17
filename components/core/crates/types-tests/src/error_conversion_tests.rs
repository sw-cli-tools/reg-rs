use reg_rs_types::error::RegError;
use std::io;

#[test]
fn test_from_string() {
    let err: RegError = "string error".to_string().into();
    assert_eq!(err.to_string(), "string error");
}

#[test]
fn test_from_str() {
    let err: RegError = "str error".into();
    assert_eq!(err.to_string(), "str error");
}

#[test]
fn test_from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let err: RegError = io_err.into();
    assert!(err.to_string().contains("denied"));
}

#[test]
fn test_from_box_dyn_error() {
    let boxed: Box<dyn std::error::Error> = "boxed error".into();
    let err: RegError = boxed.into();
    assert_eq!(err.to_string(), "boxed error");
}

use reg_rs_types::normalize::{DiffMode, apply};

#[test]
fn test_text_mode_passthrough() {
    let result = apply("hello world", &DiffMode::Text).unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_text_mode_empty() {
    let result = apply("", &DiffMode::Text).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_display_text() {
    assert_eq!(DiffMode::Text.to_string(), "text");
}

#[test]
fn test_display_json() {
    assert_eq!(DiffMode::Json.to_string(), "json");
}

#[test]
fn test_from_str_valid() {
    assert_eq!("text".parse::<DiffMode>().unwrap(), DiffMode::Text);
    assert_eq!("json".parse::<DiffMode>().unwrap(), DiffMode::Json);
}

#[test]
fn test_from_str_invalid() {
    assert!("xml".parse::<DiffMode>().is_err());
}

#[test]
fn test_default_is_text() {
    assert_eq!(DiffMode::default(), DiffMode::Text);
}

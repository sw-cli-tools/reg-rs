use super::*;

#[test]
fn test_fail_contains_input() {
    let result = fail("error");
    assert!(result.contains("error"));
}

#[test]
fn test_pass_contains_input() {
    let result = pass("ok");
    assert!(result.contains("ok"));
}

#[test]
fn test_warn_contains_input() {
    let result = warn("caution");
    assert!(result.contains("caution"));
}

#[test]
fn test_fail_symbol_not_empty() {
    assert!(!fail_symbol().is_empty());
}

#[test]
fn test_pass_symbol_not_empty() {
    assert!(!pass_symbol().is_empty());
}

#[test]
fn test_warn_symbol_not_empty() {
    assert!(!warn_symbol().is_empty());
}

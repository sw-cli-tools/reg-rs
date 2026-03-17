use reg_rs_types::types::RegressionType;

#[test]
fn test_display_label_known_types() {
    assert_eq!(RegressionType::display_label("1"), Some("Actual exit code"));
    assert_eq!(
        RegressionType::display_label("2"),
        Some("Expected exit code")
    );
    assert_eq!(RegressionType::display_label("3"), Some("stderr add"));
    assert_eq!(RegressionType::display_label("4"), Some("stderr remove"));
    assert_eq!(RegressionType::display_label("6"), Some("stdout add"));
    assert_eq!(RegressionType::display_label("7"), Some("stdout remove"));
}

#[test]
fn test_display_label_same_types_return_none() {
    assert_eq!(RegressionType::display_label("5"), None);
    assert_eq!(RegressionType::display_label("8"), None);
}

#[test]
fn test_display_label_invalid() {
    assert_eq!(RegressionType::display_label("0"), None);
    assert_eq!(RegressionType::display_label("99"), None);
    assert_eq!(RegressionType::display_label("abc"), None);
}

#[test]
fn test_has_differences() {
    assert!(RegressionType::has_differences("1"));
    assert!(!RegressionType::has_differences("2"));
    assert!(RegressionType::has_differences("3"));
    assert!(RegressionType::has_differences("4"));
    assert!(!RegressionType::has_differences("5"));
    assert!(RegressionType::has_differences("6"));
    assert!(RegressionType::has_differences("7"));
    assert!(!RegressionType::has_differences("8"));
    assert!(!RegressionType::has_differences("abc"));
}

#[test]
fn test_from_code_valid() {
    assert!(RegressionType::from_code("1").is_some());
    assert!(RegressionType::from_code("8").is_some());
}

#[test]
fn test_from_code_invalid() {
    assert!(RegressionType::from_code("0").is_none());
    assert!(RegressionType::from_code("9").is_none());
    assert!(RegressionType::from_code("abc").is_none());
}

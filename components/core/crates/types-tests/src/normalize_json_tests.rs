use reg_rs_types::normalize::{DiffMode, apply};

#[test]
fn test_json_sorts_keys() {
    let input = r#"{"z":1,"a":2,"m":3}"#;
    let result = apply(input, &DiffMode::Json).unwrap();
    let a_pos = result.find("\"a\"").unwrap();
    let m_pos = result.find("\"m\"").unwrap();
    let z_pos = result.find("\"z\"").unwrap();
    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
}

#[test]
fn test_json_nested_sorts() {
    let input = r#"{"outer":{"z":1,"a":2}}"#;
    let result = apply(input, &DiffMode::Json).unwrap();
    let a_pos = result.find("\"a\"").unwrap();
    let z_pos = result.find("\"z\"").unwrap();
    assert!(a_pos < z_pos);
}

#[test]
fn test_json_invalid_returns_error() {
    let result = apply("not json at all", &DiffMode::Json);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not valid JSON"));
}

#[test]
fn test_json_empty_passthrough() {
    let result = apply("", &DiffMode::Json).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_json_whitespace_only_passthrough() {
    let result = apply("  \n  ", &DiffMode::Json).unwrap();
    assert_eq!(result, "  \n  ");
}

#[test]
fn test_json_deterministic() {
    let input1 = r#"{"b": 2, "a": 1}"#;
    let input2 = r#"{"a":1,"b":2}"#;
    let result1 = apply(input1, &DiffMode::Json).unwrap();
    let result2 = apply(input2, &DiffMode::Json).unwrap();
    assert_eq!(result1, result2);
}

use super::*;

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
fn test_json_sorts_keys() {
    let input = r#"{"z":1,"a":2,"m":3}"#;
    let result = apply(input, &DiffMode::Json).unwrap();
    // Keys should be sorted: a, m, z
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

#[test]
fn test_lines_unordered_sorts() {
    let input = "cherry\napple\nbanana\n";
    let result = apply(input, &DiffMode::LinesUnordered).unwrap();
    assert_eq!(result, "apple\nbanana\ncherry\n");
}

#[test]
fn test_lines_unordered_deterministic() {
    let input1 = "c\na\nb\n";
    let input2 = "b\nc\na\n";
    let r1 = apply(input1, &DiffMode::LinesUnordered).unwrap();
    let r2 = apply(input2, &DiffMode::LinesUnordered).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn test_lines_unordered_empty() {
    let result = apply("", &DiffMode::LinesUnordered).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_lines_unordered_no_trailing_newline() {
    let input = "b\na";
    let result = apply(input, &DiffMode::LinesUnordered).unwrap();
    assert_eq!(result, "a\nb");
}

#[test]
fn test_display_lines_unordered() {
    assert_eq!(DiffMode::LinesUnordered.to_string(), "lines-unordered");
}

#[test]
fn test_from_str_lines_unordered() {
    assert_eq!(
        "lines-unordered".parse::<DiffMode>().unwrap(),
        DiffMode::LinesUnordered
    );
}

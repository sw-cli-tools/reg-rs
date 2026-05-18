use reg_rs_config::time;

#[test]
fn test_now_format() {
    let result = time::now();
    // Should match YYYY-MM-DDTHH:MM:SS
    assert_eq!(result.len(), 19, "expected 19 chars: {result}");
    assert_eq!(&result[4..5], "-");
    assert_eq!(&result[7..8], "-");
    assert_eq!(&result[10..11], "T");
    assert_eq!(&result[13..14], ":");
    assert_eq!(&result[16..17], ":");
}

#[test]
fn test_now_returns_different_from_empty() {
    let result = time::now();
    assert!(!result.is_empty());
}

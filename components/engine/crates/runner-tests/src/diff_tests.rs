use reg_rs_runner::diff;

#[test]
fn test_get_differences_identical() {
    assert!(diff::get_differences("hello", "hello").is_none());
}

#[test]
fn test_get_differences_different() {
    let diffs = diff::get_differences("hello", "world");
    assert!(diffs.is_some());
    let diffs = diffs.expect("expected differences between 'hello' and 'world'");
    assert!(!diffs.is_empty());
}

#[test]
fn test_get_differences_empty_strings() {
    assert!(diff::get_differences("", "").is_none());
}

#[test]
fn test_get_differences_added_content() {
    use text_diff::Difference;
    let diffs =
        diff::get_differences("", "new content").expect("expected differences for added content");
    assert!(diffs.iter().any(|d| matches!(d, Difference::Add(_))));
}

#[test]
fn test_get_differences_removed_content() {
    use text_diff::Difference;
    let diffs =
        diff::get_differences("old content", "").expect("expected differences for removed content");
    assert!(diffs.iter().any(|d| matches!(d, Difference::Rem(_))));
}

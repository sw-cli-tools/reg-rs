use reg_rs_renderer::diff_format;
use reg_rs_renderer::templates;

#[test]
fn test_classify_difference_known_types() {
    let result =
        diff_format::classify_difference("1", "42").expect("expected classification for type 1");
    assert_eq!(result.kind, diff_format::DiffKind::Add);
    assert_eq!(result.label, "Actual exit code");

    let result =
        diff_format::classify_difference("2", "0").expect("expected classification for type 2");
    assert_eq!(result.kind, diff_format::DiffKind::Remove);
    assert_eq!(result.label, "Expected exit code");
}

#[test]
fn test_classify_difference_skips_same() {
    assert!(diff_format::classify_difference("5", "same").is_none());
    assert!(diff_format::classify_difference("8", "same").is_none());
}

#[test]
fn test_classify_difference_escapes_html() {
    let result = diff_format::classify_difference("6", "<b>xss</b>")
        .expect("expected classification for type 6");
    assert!(result.value.contains("&lt;b&gt;"));
    assert!(!result.value.contains("<b>xss"));
}

#[test]
fn test_highlight_diff_marks_changed_chars() {
    let (old_html, new_html) = diff_format::highlight_diff("abc123def", "abc456def");
    assert!(old_html.contains("<mark>123</mark>"));
    assert!(new_html.contains("<mark>456</mark>"));
}

#[test]
fn test_highlight_diff_identical() {
    let (old_html, new_html) = diff_format::highlight_diff("same", "same");
    assert!(!old_html.contains("<mark>"));
    assert!(!new_html.contains("<mark>"));
}

#[test]
fn test_html_escape() {
    let escaped = diff_format::html_escape("<script>alert('xss')</script>");
    assert!(escaped.contains("&lt;"));
    assert!(escaped.contains("&gt;"));
    assert!(!escaped.contains("<script>"));
}

#[test]
fn test_categorize_runs() {
    let runs = vec![
        templates::TestDetails {
            created: "2024-01-01".to_string(),
            diffs: Some(vec!["diff".to_string()]),
            last_ran: Some("2024-01-02".to_string()),
            name: "failed_test".to_string(),
        },
        templates::TestDetails {
            created: "2024-01-01".to_string(),
            diffs: None,
            last_ran: Some("2024-01-02".to_string()),
            name: "passed_test".to_string(),
        },
        templates::TestDetails {
            created: "2024-01-01".to_string(),
            diffs: None,
            last_ran: None,
            name: "pending_test".to_string(),
        },
    ];
    let (failed, passed, not_run) = templates::categorize_runs(&runs);
    assert_eq!(failed, vec!["failed_test"]);
    assert_eq!(passed, vec!["passed_test"]);
    assert_eq!(not_run, vec!["pending_test"]);
}

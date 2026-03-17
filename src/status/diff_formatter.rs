/// Diff formatting utilities for status page

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DiffKind {
    /// Added (actual/new)
    Add,
    /// Removed (expected/baseline)
    Remove,
}

/// Result of classifying a difference
#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    /// Kind of difference
    pub kind: DiffKind,
    /// Label for display
    pub label: &'static str,
    /// Escaped value
    pub value: String,
}

/// Classify a difference by type and value
pub fn classify_difference(type_code: &str, value: &str) -> Option<DiffResult> {
    let escaped = html_escape(value);
    match crate::diff::RegressionType::from_code(type_code)? {
        crate::diff::RegressionType::ActualCode => Some(DiffResult {
            kind: DiffKind::Add,
            label: "Actual exit code",
            value: escaped,
        }),
        crate::diff::RegressionType::ExpectedCode => Some(DiffResult {
            kind: DiffKind::Remove,
            label: "Expected exit code",
            value: escaped,
        }),
        crate::diff::RegressionType::StderrAdd => Some(DiffResult {
            kind: DiffKind::Add,
            label: "Stderr",
            value: escaped,
        }),
        crate::diff::RegressionType::StderrRemove => Some(DiffResult {
            kind: DiffKind::Remove,
            label: "Stderr",
            value: escaped,
        }),
        crate::diff::RegressionType::StdoutAdd => Some(DiffResult {
            kind: DiffKind::Add,
            label: "Stdout",
            value: escaped,
        }),
        crate::diff::RegressionType::StdoutRemove => Some(DiffResult {
            kind: DiffKind::Remove,
            label: "Stdout",
            value: escaped,
        }),
        crate::diff::RegressionType::StderrSame | crate::diff::RegressionType::StdoutSame => None,
    }
}

/// Format diffs into HTML with inline character highlighting
pub fn format_diffs_html(diffs: &[(String, String)]) -> Vec<String> {
    let classified: Vec<_> = diffs
        .iter()
        .filter_map(|(code, value)| classify_difference(code, value))
        .collect();

    let mut result = Vec::new();
    let mut i = 0;
    while i < classified.len() {
        let diff_result = &classified[i];
        if diff_result.kind == DiffKind::Remove
            && i + 1 < classified.len()
            && classified[i + 1].kind == DiffKind::Add
        {
            let add_result = &classified[i + 1];
            let (hl_old, hl_new) = highlight_diff(&diff_result.value, &add_result.value);
            result.push(format!(
                r#"<div class="diff-pair"><div class="diff-label">{}</div><div class="diff-remove">- expected: {}</div><div class="diff-add">+ actual:&nbsp;&nbsp; {}</div></div>"#,
                diff_result.label,
                hl_old,
                hl_new
            ));
            i += 2;
        } else {
            let (class, prefix) = match diff_result.kind {
                DiffKind::Add => ("diff-add", "+ actual"),
                DiffKind::Remove => ("diff-remove", "- expected"),
            };
            result.push(format!(
                r#"<div class="{}">{} ({}): {}</div>"#,
                class, prefix, diff_result.label, diff_result.value
            ));
            i += 1;
        }
    }
    result
}

/// Highlight character-level differences between two strings
pub fn highlight_diff(old: &str, new: &str) -> (String, String) {
    let prefix_len = find_common_prefix_len(old, new);
    let suffix_len = find_common_suffix_len(old, new, prefix_len);

    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();

    let old_mid_end = old_chars.len().saturating_sub(suffix_len);
    let new_mid_end = new_chars.len().saturating_sub(suffix_len);

    let old_prefix: String = old_chars[..prefix_len].iter().collect();
    let old_mid: String = old_chars[prefix_len..old_mid_end].iter().collect();
    let old_suffix: String = old_chars[old_mid_end..].iter().collect();

    let new_prefix: String = new_chars[..prefix_len].iter().collect();
    let new_mid: String = new_chars[prefix_len..new_mid_end].iter().collect();
    let new_suffix: String = new_chars[new_mid_end..].iter().collect();

    if old_mid.is_empty() && new_mid.is_empty() {
        (old.to_string(), new.to_string())
    } else {
        (
            format!("{}<mark>{}</mark>{}", old_prefix, old_mid, old_suffix),
            format!("{}<mark>{}</mark>{}", new_prefix, new_mid, new_suffix),
        )
    }
}

/// Find common prefix length between two strings
fn find_common_prefix_len(old: &str, new: &str) -> usize {
    old.chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .count()
}

/// Find common suffix length between two strings
fn find_common_suffix_len(old: &str, new: &str, prefix_len: usize) -> usize {
    let old_rest: Vec<char> = old.chars().skip(prefix_len).collect();
    let new_rest: Vec<char> = new.chars().skip(prefix_len).collect();
    old_rest
        .iter()
        .rev()
        .zip(new_rest.iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
}

/// Escape HTML special characters to prevent XSS
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_difference_known_types() {
        let result = classify_difference("1", "42").unwrap();
        assert_eq!(result.kind, DiffKind::Add);
        assert_eq!(result.label, "Actual exit code");

        let result = classify_difference("2", "0").unwrap();
        assert_eq!(result.kind, DiffKind::Remove);
        assert_eq!(result.label, "Expected exit code");
    }

    #[test]
    fn test_classify_difference_skips_same() {
        assert!(classify_difference("5", "same").is_none());
        assert!(classify_difference("8", "same").is_none());
    }

    #[test]
    fn test_classify_difference_unknown() {
        assert!(classify_difference("99", "data").is_none());
        assert!(classify_difference("abc", "data").is_none());
    }

    #[test]
    fn test_classify_difference_escapes_html() {
        let result = classify_difference("6", "<b>xss</b>").unwrap();
        assert!(result.value.contains("&lt;b&gt;"));
        assert!(!result.value.contains("<b>xss"));
    }

    #[test]
    fn test_format_diffs_html_pairs_remove_add() {
        let diffs = vec![
            ("7".to_string(), "old_value".to_string()),
            ("6".to_string(), "new_value".to_string()),
        ];
        let result = format_diffs_html(&diffs);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("diff-pair"));
        assert!(result[0].contains("expected"));
        assert!(result[0].contains("actual"));
    }

    #[test]
    fn test_highlight_diff_marks_changed_chars() {
        let (old_html, new_html) = highlight_diff("abc123def", "abc456def");
        assert!(old_html.contains("<mark>123</mark>"));
        assert!(new_html.contains("<mark>456</mark>"));
    }

    #[test]
    fn test_highlight_diff_identical() {
        let (old_html, new_html) = highlight_diff("same", "same");
        assert!(!old_html.contains("<mark>"));
        assert!(!new_html.contains("<mark>"));
    }

    #[test]
    fn test_html_escape() {
        let escaped = html_escape("<script>alert('xss')</script>");
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&gt;"));
        assert!(!escaped.contains("<script>"));
    }
}

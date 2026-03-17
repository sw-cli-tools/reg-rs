/// Diff formatting utilities for status page
use reg_rs_types::types::RegressionType;

/// Kind of difference (added or removed)
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
    match RegressionType::from_code(type_code)? {
        RegressionType::ActualCode => Some(DiffResult {
            kind: DiffKind::Add,
            label: "Actual exit code",
            value: escaped,
        }),
        RegressionType::ExpectedCode => Some(DiffResult {
            kind: DiffKind::Remove,
            label: "Expected exit code",
            value: escaped,
        }),
        RegressionType::StderrAdd => Some(DiffResult {
            kind: DiffKind::Add,
            label: "Stderr",
            value: escaped,
        }),
        RegressionType::StderrRemove => Some(DiffResult {
            kind: DiffKind::Remove,
            label: "Stderr",
            value: escaped,
        }),
        RegressionType::StdoutAdd => Some(DiffResult {
            kind: DiffKind::Add,
            label: "Stdout",
            value: escaped,
        }),
        RegressionType::StdoutRemove => Some(DiffResult {
            kind: DiffKind::Remove,
            label: "Stdout",
            value: escaped,
        }),
        RegressionType::StderrSame | RegressionType::StdoutSame => None,
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
                diff_result.label, hl_old, hl_new
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

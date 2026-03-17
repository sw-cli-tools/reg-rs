use reg_rs_types::normalize::{DiffMode, apply};

#[test]
fn test_lines_unordered_sorts() {
    let input = "cherry\napple\nbanana\n";
    let result = apply(input, &DiffMode::LinesUnordered)
        .expect("failed to apply lines-unordered normalization");
    assert_eq!(result, "apple\nbanana\ncherry\n");
}

#[test]
fn test_lines_unordered_deterministic() {
    let input1 = "c\na\nb\n";
    let input2 = "b\nc\na\n";
    let r1 = apply(input1, &DiffMode::LinesUnordered)
        .expect("failed to apply lines-unordered normalization");
    let r2 = apply(input2, &DiffMode::LinesUnordered)
        .expect("failed to apply lines-unordered normalization");
    assert_eq!(r1, r2);
}

#[test]
fn test_lines_unordered_empty() {
    let result = apply("", &DiffMode::LinesUnordered)
        .expect("failed to apply lines-unordered normalization");
    assert_eq!(result, "");
}

#[test]
fn test_lines_unordered_no_trailing_newline() {
    let input = "b\na";
    let result = apply(input, &DiffMode::LinesUnordered)
        .expect("failed to apply lines-unordered normalization");
    assert_eq!(result, "a\nb");
}

#[test]
fn test_display_lines_unordered() {
    assert_eq!(DiffMode::LinesUnordered.to_string(), "lines-unordered");
}

#[test]
fn test_from_str_lines_unordered() {
    assert_eq!(
        "lines-unordered"
            .parse::<DiffMode>()
            .expect("failed to parse diff mode"),
        DiffMode::LinesUnordered
    );
}

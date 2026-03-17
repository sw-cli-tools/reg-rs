use reg_rs_report::summary;

#[test]
fn test_summary_render() {
    let ctx = summary::SummaryReportContext::new(1, 2, 3, "my_test", 6);
    let output = summary::render(&ctx).unwrap();
    assert!(output.contains("reg-rs Summary Report"));
    assert!(output.contains("failed"));
    assert!(output.contains("not yet run"));
    assert!(output.contains("passed"));
    assert!(output.contains("my_test"));
}

#[test]
fn test_summary_render_zero_counts() {
    let ctx = summary::SummaryReportContext::new(0, 0, 0, "empty", 0);
    let output = summary::render(&ctx).unwrap();
    assert!(output.contains("empty"));
}

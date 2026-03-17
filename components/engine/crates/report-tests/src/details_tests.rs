use reg_rs_report::details;

#[test]
fn test_render_all_categories() {
    let ctx = details::DetailsReportContext::new(
        vec!["failed_test".to_string()],
        false,
        false,
        false,
        vec!["pending_test".to_string()],
        vec!["passed_test".to_string()],
    );
    let output = details::render(&ctx).unwrap();
    assert!(output.contains("failed_test"));
    assert!(output.contains("pending_test"));
    assert!(output.contains("passed_test"));
}

#[test]
fn test_render_no_failures() {
    let ctx = details::DetailsReportContext::new(
        vec![],
        true,
        false,
        false,
        vec!["pending".to_string()],
        vec!["ok".to_string()],
    );
    let output = details::render(&ctx).unwrap();
    assert!(output.contains("No Failed Tests"));
}

#[test]
fn test_render_no_passes() {
    let ctx = details::DetailsReportContext::new(
        vec!["broken".to_string()],
        false,
        true,
        true,
        vec![],
        vec![],
    );
    let output = details::render(&ctx).unwrap();
    assert!(output.contains("No Passed Tests"));
}

#[test]
fn test_render_empty() {
    let ctx = details::DetailsReportContext::new(vec![], true, true, true, vec![], vec![]);
    let output = details::render(&ctx).unwrap();
    assert!(output.contains("No Failed Tests"));
    assert!(output.contains("No Passed Tests"));
}

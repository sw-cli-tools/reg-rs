use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::status::server;
use crate::templates::views;

/// Status Counts Data
#[derive(Serialize)]
pub struct StatusCounts {
    /// Fail count
    pub fail_count: String,
    /// Not-run count
    pub not_run_count: String,
    /// Pass count
    pub pass_count: String,
    /// Test count
    pub test_count: String,
}

/// Status flags
#[derive(Serialize)]
pub struct StatusFlags {
    /// No failed tests
    pub no_failed_tests: bool,
    /// No not-yet-run tests
    pub no_not_yet_run_tests: bool,
    /// No passed tests
    pub no_passed_tests: bool,
}

/// Status view template data
#[derive(Serialize)]
pub struct StatusViewContext<'a> {
    server_started: String,
    state_updated: String,
    status_counts: StatusCounts,
    status_flags: StatusFlags,
    test_pattern: String,
    test_runs: &'a [server::TestDetails],
}

impl<'a> StatusViewContext<'a> {
    /// build Status view template data
    pub fn new(
        server_started: String,
        state_updated: String,
        status_counts: StatusCounts,
        status_flags: StatusFlags,
        test_pattern: String,
        test_runs: &'a [server::TestDetails],
    ) -> Self {
        StatusViewContext {
            server_started,
            state_updated,
            status_counts,
            status_flags,
            test_pattern,
            test_runs,
        }
    }
}

/// Render status view template
pub fn render(status_view_context: &StatusViewContext) -> crate::error::Result<String> {
    log::info!("status/render");
    let mut tt = TinyTemplate::new();
    tt.add_template("status_view_template", views::STATUS_VIEW_TEMPLATE)?;
    let rendered = tt.render("status_view_template", status_view_context)?;
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_runs() {
        let ctx = StatusViewContext::new(
            "2024-01-01T00:00:00".to_string(),
            "2024-01-01T00:00:01".to_string(),
            StatusCounts {
                fail_count: " 00000".to_string(),
                not_run_count: " 00000".to_string(),
                pass_count: " 00000".to_string(),
                test_count: " 00000".to_string(),
            },
            StatusFlags {
                no_failed_tests: true,
                no_not_yet_run_tests: true,
                no_passed_tests: true,
            },
            "my_test".to_string(),
            &[],
        );
        let output = render(&ctx).unwrap();
        assert!(output.contains("reg-rs"));
        assert!(output.contains("my_test"));
        assert!(output.contains("No failed tests"));
        assert!(output.contains("No passed tests"));
    }

    #[test]
    fn test_render_with_passed_test() {
        let runs = vec![server::TestDetails {
            created: "2024-01-01".to_string(),
            diffs: None,
            last_ran: Some("2024-01-02".to_string()),
            name: "passing_test".to_string(),
        }];
        let ctx = StatusViewContext::new(
            "2024-01-01T00:00:00".to_string(),
            "2024-01-01T00:00:01".to_string(),
            StatusCounts {
                fail_count: " 00000".to_string(),
                not_run_count: " 00000".to_string(),
                pass_count: " 00001".to_string(),
                test_count: " 00001".to_string(),
            },
            StatusFlags {
                no_failed_tests: true,
                no_not_yet_run_tests: true,
                no_passed_tests: false,
            },
            "test".to_string(),
            &runs,
        );
        let output = render(&ctx).unwrap();
        assert!(output.contains("passing_test"));
        assert!(output.contains("Passes"));
    }

    #[test]
    fn test_render_with_failed_test() {
        let runs = vec![server::TestDetails {
            created: "2024-01-01".to_string(),
            diffs: Some(vec!["+ stdout add: changed".to_string()]),
            last_ran: Some("2024-01-02".to_string()),
            name: "failing_test".to_string(),
        }];
        let ctx = StatusViewContext::new(
            "2024-01-01T00:00:00".to_string(),
            "2024-01-01T00:00:01".to_string(),
            StatusCounts {
                fail_count: " 00001".to_string(),
                not_run_count: " 00000".to_string(),
                pass_count: " 00000".to_string(),
                test_count: " 00001".to_string(),
            },
            StatusFlags {
                no_failed_tests: false,
                no_not_yet_run_tests: true,
                no_passed_tests: true,
            },
            "test".to_string(),
            &runs,
        );
        let output = render(&ctx).unwrap();
        assert!(output.contains("failing_test"));
        assert!(output.contains("Failures"));
    }
}

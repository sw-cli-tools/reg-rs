//! CLI output formatting and report generation for reg-rs.
#![deny(warnings, missing_docs)]

/// Details report: test names by category, failure details, passes
pub mod details;
/// Failure details: individual failure entries, difference types, metadata
mod details_failures;
/// Pass details: individual pass entries
mod details_passes;
/// Color formatting utilities
pub mod format;
/// Output helpers: report generation and test categorization
pub mod output;
/// Summary report: counts of pass/fail/not-yet-run
pub mod summary;

//! Pure HTML/CSS rendering for reg-rs status pages.
#![deny(warnings, missing_docs)]

/// CSS and JS assets as const strings
pub mod assets;
/// Diff formatting utilities for HTML display
pub mod diff_format;
/// HTML page templates (landing, status dashboard)
pub mod templates;
/// Test run state collection and categorization
pub mod test_runner;

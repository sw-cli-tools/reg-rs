//! Modification commands: remove, rebase, reset, migrate, status.
#![deny(warnings, missing_docs)]

/// Migrate .tdb to .rgt format
pub mod migrate;
/// Rebase tests to accept latest output
pub mod rebase;
/// Remove test files
pub mod remove;
/// Reset latest results
pub mod reset;
/// Status monitoring web server
pub mod status;

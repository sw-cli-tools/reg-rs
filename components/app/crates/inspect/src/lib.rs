//! Read-only inspection commands: show, list, complete.
#![deny(warnings, missing_docs)]

/// Shell completion output
pub mod complete;
/// List tests with status
pub mod list;
/// Show test details (dispatcher)
pub mod show;
/// Show .rgt test details
pub mod show_rgt;
/// Show .tdb test details
pub mod show_tdb;

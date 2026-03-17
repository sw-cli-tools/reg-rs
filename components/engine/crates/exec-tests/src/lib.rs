//! Tests for reg-rs-exec crate
#![deny(warnings, missing_docs)]

/// Mock command executor for testing
pub mod mock;

#[cfg(test)]
mod executor_tests;
#[cfg(test)]
mod process_tests;

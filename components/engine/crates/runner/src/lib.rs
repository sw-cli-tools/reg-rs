//! Test orchestration, execution, and diff pipeline for reg-rs.
#![deny(warnings, missing_docs)]

/// Diff computation: compare test outputs and detect regressions
pub mod diff;
/// Store computed differences in the database
pub mod diff_store;
/// Dispatch: route .rgt vs .tdb tests to appropriate handlers
pub mod dispatch;
/// Test runner: execute tests and collect results
pub mod runner;
/// Parallel and sequential test execution strategies
pub mod runner_parallel;

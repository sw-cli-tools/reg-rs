//! Configuration, logging, and time utilities for reg-rs CLI.
#![deny(warnings, missing_docs)]

/// Configuration builder from CLI args
pub mod builder;
/// Configuration struct and option extraction
pub mod config;
/// File logging setup
pub mod logging;
/// Timestamp formatting
pub mod time;

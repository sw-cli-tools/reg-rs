//! modules
#![deny(warnings, missing_docs)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
use std::env;

lazy_static! {
    static ref DEBUG: bool = env::args().any(|s| s.starts_with("-d"));
}

macro_rules! md {
    // maybe-debug
    ( $e:expr ) => {
        if *crate::DEBUG {
            dbg!($e);
        }
    };
}

/// Default status port for web
pub const DEFAULT_STATUS_PORT: u16 = 4111;
/// Status banner
pub const STATUS_BANNER: &str = "RTT1 Status Server";

/// Argument parsing
pub mod args;
/// Test Builder
pub mod builder;
/// Command Processor
pub mod command;
/// Command Configuration
pub mod config;
/// Test Results Database 
pub mod db;
/// Test Differences
pub mod diff;
/// Error types and utilities
pub mod error;
/// Test Finder
pub mod finder;
/// Logging
pub mod logging;
/// Test Process
pub mod process;
/// Test Result DB Queries
pub mod queries;
/// Test Result Reports
pub mod reporters;
/// Test Runner
pub mod runner;
/// Test DB Interface
pub mod sqlite;
/// Status
pub mod status;
/// Templates
pub mod templates;
/// Time
pub mod time;
/// Utility
pub mod util;

//! Core types, error handling, constants, and normalization for reg-rs.
#![deny(warnings, missing_docs)]

/// Constants used across reg-rs components
pub mod constants;
/// Error types and utilities
pub mod error;
/// Built-in output normalization for diff modes (text, json, lines-unordered)
pub mod normalize;
/// Pure data types for test results and regression tracking
pub mod types;

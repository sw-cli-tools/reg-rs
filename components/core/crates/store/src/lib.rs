//! Persistence layer for reg-rs: SQLite storage and file locking.
#![deny(warnings, missing_docs)]

/// High-level database operations with file locking (results)
pub mod db;
/// High-level database operations for metadata and latest results
pub mod db_meta;
/// High-level database operations with file locking (differences)
pub mod db_ops;
/// SQL statement template rendering
pub mod queries;
/// Low-level SQLite read/write operations for test results
pub mod sqlite;
/// Low-level SQLite read/write operations for differences and metadata
pub mod sqlite_diff;

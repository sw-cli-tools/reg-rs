//! Process execution and dependency injection for reg-rs.
#![deny(warnings, missing_docs)]

/// Command executor trait for dependency injection
pub mod executor;
/// Output preprocessing via external shell commands
pub mod preprocess;
/// Process spawning, output capture, and timeout handling
pub mod process;
/// Process I/O helpers (output collection, child process cleanup)
pub mod process_io;

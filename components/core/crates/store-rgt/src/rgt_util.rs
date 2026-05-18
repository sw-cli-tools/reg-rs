//! Utility functions for .rgt file path resolution.

use reg_rs_types::constants::{RGT_EXTENSION, TDB_EXTENSION};

/// Get the `.tdb` cache path for an `.rgt` file.
pub fn tdb_path_for_rgt(rgt_path: &str) -> String {
    let p = std::path::Path::new(rgt_path);
    p.with_extension(TDB_EXTENSION)
        .to_string_lossy()
        .to_string()
}

/// Check if a path ends with the `.rgt` extension.
pub fn is_rgt_path(path: &str) -> bool {
    path.ends_with(&format!(".{RGT_EXTENSION}"))
}

/// Resolve the .tdb database path for a test file.
///
/// For `.rgt` files, returns the companion `.tdb` cache path.
/// For `.tdb` files, returns the path as-is.
pub fn db_path(test_path: &str) -> String {
    if test_path.ends_with(&format!(".{RGT_EXTENSION}")) {
        tdb_path_for_rgt(test_path)
    } else {
        test_path.to_string()
    }
}

use reg_rs_store::db;
use reg_rs_store_rgt::rgt;
use reg_rs_types::constants::{RGT_EXTENSION, TDB_EXTENSION};
use reg_rs_types::error::Result;

/// Resolve a test name to a full `.rgt` path in the data directory.
///
/// Bare names (no directory component) are placed in the data directory.
/// Paths with directories are used as-is. Extensions `.tdb` and `.rgt`
/// are normalized to `.rgt`.
pub fn resolve_test_path(test: &str) -> String {
    let path = std::path::Path::new(test);
    let mut resolved = if path.parent().is_some_and(|p| p != std::path::Path::new("")) {
        path.to_path_buf()
    } else {
        reg_rs_discover::data_dir::data_dir().join(test)
    };
    if resolved
        .extension()
        .is_some_and(|ext| ext == TDB_EXTENSION || ext == RGT_EXTENSION)
    {
        resolved.set_extension("");
    }
    let mut name = resolved.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{RGT_EXTENSION}"));
    resolved.set_file_name(name);
    resolved.to_string_lossy().to_string()
}

/// Read the command and database path for a test file.
///
/// For `.rgt` tests, reads the spec from the TOML file.
/// For `.tdb` tests, reads from the database.
pub fn read_test_command(test_path: &str) -> Result<(String, String)> {
    if test_path.ends_with(&format!(".{RGT_EXTENSION}")) {
        let spec = rgt::parse_rgt(test_path)?;
        let tdb_path = rgt::tdb_path_for_rgt(test_path);
        Ok((spec.command, tdb_path))
    } else {
        let original = db::read_original_results(test_path)?;
        Ok((original.command, test_path.to_string()))
    }
}

/// Format a test path as a display name (file stem only).
pub fn format_test_name(test_path: &str) -> String {
    std::path::Path::new(test_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

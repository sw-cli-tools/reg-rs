use std::collections::HashMap;

use walkdir::{DirEntry, WalkDir};

use reg_rs_types::constants::{RGT_EXTENSION, TDB_EXTENSION};
use reg_rs_types::error::{RegError, Result};

/// Test Names data
#[derive(Debug)]
pub struct TestNames {
    /// Names that matched pattern
    pub found: Vec<String>,
}

/// Result of test discovery, including the directory that was searched.
#[derive(Debug)]
pub struct DiscoverResult {
    /// Paths of matched test files (.rgt preferred over .tdb)
    pub found: Vec<String>,
    /// The data directory that was searched
    pub data_dir: std::path::PathBuf,
}

/// Determine if a directory entry is a hidden file
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| s.starts_with('.'))
}

/// Check if a file has a recognized test extension (.rgt or .tdb).
fn is_test_file(path: &str) -> bool {
    path.ends_with(&format!(".{}", TDB_EXTENSION)) || path.ends_with(&format!(".{}", RGT_EXTENSION))
}

/// Discover tests matching a substring pattern in the data directory.
///
/// Returns both the matched test paths and the data directory that was searched,
/// so callers can include the path in diagnostic messages.
///
/// When both `.rgt` and `.tdb` files exist for the same test stem, the `.rgt`
/// file takes precedence.
pub fn discover(pattern: String) -> Result<DiscoverResult> {
    log::info!("finder/discover pattern {}", &pattern);
    let data_dir = crate::data_dir::data_dir();
    if !data_dir.exists() {
        return Err(RegError::Other(format!(
            "data directory does not exist: {}\n\
             hint: set REG_RS_DATA_DIR to the directory containing your .tdb or .rgt files",
            data_dir.display()
        )));
    }
    let names = discover_in(&data_dir, &pattern)?;
    Ok(DiscoverResult {
        found: names.found,
        data_dir,
    })
}

/// Discover tests matching a substring pattern in a specific directory.
///
/// Finds both `.rgt` and `.tdb` files. When both exist for the same stem,
/// the `.rgt` file wins. The special patterns ".tdb" and ".rgt" are treated
/// as "match all tests" for backward compatibility.
pub fn discover_in(dir: &std::path::Path, pattern: &str) -> Result<TestNames> {
    // Canonicalize to resolve symlinks (e.g., /var -> /private/var on macOS)
    let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());

    // Treat ".tdb" and ".rgt" patterns as "match all" for backward compatibility
    let match_all = pattern == ".tdb" || pattern == ".rgt";

    let all_files: Vec<String> = WalkDir::new(&dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .map(|e| e.path().display().to_string())
        .filter(|path| is_test_file(path) && (match_all || path.contains(pattern)))
        .collect();

    // Deduplicate: when both .rgt and .tdb exist for the same stem, prefer .rgt
    let mut by_stem: HashMap<String, String> = HashMap::new();
    for path in all_files {
        let p = std::path::Path::new(&path);
        let stem = p.with_extension("").to_string_lossy().to_string();
        let is_rgt = path.ends_with(&format!(".{}", RGT_EXTENSION));
        by_stem
            .entry(stem)
            .and_modify(|existing| {
                // .rgt takes precedence over .tdb
                if is_rgt {
                    *existing = path.clone();
                }
            })
            .or_insert(path);
    }

    let mut found: Vec<String> = by_stem.into_values().collect();
    found.sort();
    log::debug!("finder/discover_in found {:?}", &found);
    Ok(TestNames { found })
}

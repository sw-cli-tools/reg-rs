use std::collections::HashMap;

use walkdir::{DirEntry, WalkDir};

/// Test Names data
#[derive(Debug)]
pub struct TestNames {
    /// Names that matched pattern
    pub found: Vec<String>,
}

/// determine if a directory entry is a hidden file
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| s.starts_with('.'))
}

/// Check if a file has a recognized test extension (.rgt or .tdb).
fn is_test_file(path: &str) -> bool {
    path.ends_with(&format!(".{}", crate::TDB_EXTENSION))
        || path.ends_with(&format!(".{}", crate::rgt::RGT_EXTENSION))
}

/// Discover tests matching a substring pattern in the data directory.
///
/// Returns both the matched test paths and the data directory that was searched,
/// so callers can include the path in diagnostic messages.
///
/// When both `.rgt` and `.tdb` files exist for the same test stem, the `.rgt`
/// file takes precedence.
pub fn discover(pattern: String) -> crate::error::Result<DiscoverResult> {
    log::info!("finder/discover pattern {}", &pattern);
    let data_dir = crate::data_dir();
    if !data_dir.exists() {
        return Err(crate::error::RegError::Other(format!(
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

/// Result of test discovery, including the directory that was searched.
#[derive(Debug)]
pub struct DiscoverResult {
    /// Paths of matched test files (.rgt preferred over .tdb)
    pub found: Vec<String>,
    /// The data directory that was searched
    pub data_dir: std::path::PathBuf,
}

/// Discover tests matching a substring pattern in a specific directory.
///
/// Finds both `.rgt` and `.tdb` files. When both exist for the same stem,
/// the `.rgt` file wins. The special patterns ".tdb" and ".rgt" are treated
/// as "match all tests" for backward compatibility.
fn discover_in(dir: &std::path::Path, pattern: &str) -> crate::error::Result<TestNames> {
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
        let is_rgt = path.ends_with(&format!(".{}", crate::rgt::RGT_EXTENSION));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a test directory with a non-hidden name (tempfile creates
    /// directories starting with `.tmp` on macOS, which `is_hidden` filters out).
    fn test_dir() -> (tempfile::TempDir, std::path::PathBuf) {
        let parent = tempfile::tempdir().unwrap();
        let dir = parent.path().join("testdata");
        fs::create_dir(&dir).unwrap();
        (parent, dir)
    }

    #[test]
    fn test_discover_finds_matching_tdb_files() {
        let (_parent, dir) = test_dir();
        fs::write(dir.join("hello.tdb"), "").unwrap();
        fs::write(dir.join("world.tdb"), "").unwrap();
        fs::write(dir.join("other.txt"), "").unwrap();

        let result = discover_in(&dir, ".tdb").unwrap();
        assert_eq!(result.found.len(), 2);

        let result = discover_in(&dir, "hello").unwrap();
        assert_eq!(result.found.len(), 1);
        assert!(result.found[0].ends_with("hello.tdb"));
    }

    #[test]
    fn test_discover_finds_rgt_files() {
        let (_parent, dir) = test_dir();
        fs::write(dir.join("hello.rgt"), "").unwrap();
        fs::write(dir.join("world.rgt"), "").unwrap();

        let result = discover_in(&dir, ".rgt").unwrap();
        assert_eq!(result.found.len(), 2);

        let result = discover_in(&dir, "hello").unwrap();
        assert_eq!(result.found.len(), 1);
        assert!(result.found[0].ends_with("hello.rgt"));
    }

    #[test]
    fn test_discover_rgt_takes_precedence_over_tdb() {
        let (_parent, dir) = test_dir();
        fs::write(dir.join("hello.rgt"), "").unwrap();
        fs::write(dir.join("hello.tdb"), "").unwrap();
        fs::write(dir.join("world.tdb"), "").unwrap();

        let result = discover_in(&dir, ".tdb").unwrap();
        assert_eq!(result.found.len(), 2);
        // hello should resolve to .rgt, world to .tdb
        assert!(result.found.iter().any(|p| p.ends_with("hello.rgt")));
        assert!(result.found.iter().any(|p| p.ends_with("world.tdb")));
    }

    #[test]
    fn test_discover_tdb_pattern_matches_all() {
        let (_parent, dir) = test_dir();
        fs::write(dir.join("a.rgt"), "").unwrap();
        fs::write(dir.join("b.tdb"), "").unwrap();

        // ".tdb" pattern should match both .rgt and .tdb files
        let result = discover_in(&dir, ".tdb").unwrap();
        assert_eq!(result.found.len(), 2);
    }

    #[test]
    fn test_discover_matches_by_substring() {
        let (_parent, dir) = test_dir();
        fs::write(dir.join("test_a.tdb"), "").unwrap();
        fs::write(dir.join("test_b.tdb"), "").unwrap();
        fs::write(dir.join("other.tdb"), "").unwrap();

        let result = discover_in(&dir, "test_").unwrap();
        assert_eq!(result.found.len(), 2);
    }

    #[test]
    fn test_discover_ignores_non_test_files() {
        let (_parent, dir) = test_dir();
        fs::write(dir.join("foo.txt"), "").unwrap();
        fs::write(dir.join("bar.json"), "").unwrap();

        let result = discover_in(&dir, "foo").unwrap();
        assert!(result.found.is_empty());
    }

    #[test]
    fn test_discover_ignores_hidden_directories() {
        let (_parent, dir) = test_dir();
        let hidden_dir = dir.join(".hidden");
        fs::create_dir(&hidden_dir).unwrap();
        fs::write(hidden_dir.join("secret.tdb"), "").unwrap();
        fs::write(dir.join("visible.tdb"), "").unwrap();

        let result = discover_in(&dir, ".tdb").unwrap();
        assert_eq!(result.found.len(), 1);
        assert!(result.found[0].contains("visible"));
    }

    #[test]
    fn test_discover_no_matches_returns_empty() {
        let (_parent, dir) = test_dir();
        fs::write(dir.join("foo.tdb"), "").unwrap();

        let result = discover_in(&dir, "nonexistent").unwrap();
        assert!(result.found.is_empty());
    }

    #[test]
    fn test_discover_empty_directory() {
        let (_parent, dir) = test_dir();
        let result = discover_in(&dir, "anything").unwrap();
        assert!(result.found.is_empty());
    }
}

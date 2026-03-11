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

/// Discover tests matching a substring pattern in the data directory.
pub fn discover(pattern: String) -> crate::error::Result<TestNames> {
    log::info!("finder/discover pattern {}", &pattern);
    let data_dir = crate::data_dir();
    discover_in(&data_dir, &pattern)
}

/// Discover tests matching a substring pattern in a specific directory.
fn discover_in(dir: &std::path::Path, pattern: &str) -> crate::error::Result<TestNames> {
    // Canonicalize to resolve symlinks (e.g., /var -> /private/var on macOS)
    let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let found: Vec<String> = WalkDir::new(&dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .map(|e| e.path().display().to_string())
        .filter(|path| path.contains(pattern) && path.ends_with(".tdb"))
        .collect();
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
    fn test_discover_matches_by_substring() {
        let (_parent, dir) = test_dir();
        fs::write(dir.join("test_a.tdb"), "").unwrap();
        fs::write(dir.join("test_b.tdb"), "").unwrap();
        fs::write(dir.join("other.tdb"), "").unwrap();

        let result = discover_in(&dir, "test_").unwrap();
        assert_eq!(result.found.len(), 2);
    }

    #[test]
    fn test_discover_ignores_non_tdb_files() {
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

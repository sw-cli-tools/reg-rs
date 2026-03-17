use std::fs;

use reg_rs_discover::finder;

/// Create a test directory with a non-hidden name (tempfile creates
/// directories starting with `.tmp` on macOS, which `is_hidden` filters out).
fn test_dir() -> (tempfile::TempDir, std::path::PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let dir = parent.path().join("testdata");
    fs::create_dir(&dir).unwrap();
    (parent, dir)
}

#[test]
fn test_discover_ignores_non_test_files() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("foo.txt"), "").unwrap();
    fs::write(dir.join("bar.json"), "").unwrap();

    let result = finder::discover_in(&dir, "foo").unwrap();
    assert!(result.found.is_empty());
}

#[test]
fn test_discover_ignores_hidden_directories() {
    let (_parent, dir) = test_dir();
    let hidden_dir = dir.join(".hidden");
    fs::create_dir(&hidden_dir).unwrap();
    fs::write(hidden_dir.join("secret.tdb"), "").unwrap();
    fs::write(dir.join("visible.tdb"), "").unwrap();

    let result = finder::discover_in(&dir, ".tdb").unwrap();
    assert_eq!(result.found.len(), 1);
    assert!(result.found[0].contains("visible"));
}

#[test]
fn test_discover_no_matches_returns_empty() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("foo.tdb"), "").unwrap();

    let result = finder::discover_in(&dir, "nonexistent").unwrap();
    assert!(result.found.is_empty());
}

#[test]
fn test_discover_empty_directory() {
    let (_parent, dir) = test_dir();
    let result = finder::discover_in(&dir, "anything").unwrap();
    assert!(result.found.is_empty());
}

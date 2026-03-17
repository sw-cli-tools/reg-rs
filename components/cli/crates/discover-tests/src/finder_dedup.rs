use std::fs;

use reg_rs_discover::finder;

/// Create a test directory with a non-hidden name (tempfile creates
/// directories starting with `.tmp` on macOS, which `is_hidden` filters out).
fn test_dir() -> (tempfile::TempDir, std::path::PathBuf) {
    let parent = tempfile::tempdir().expect("failed to create temp dir");
    let dir = parent.path().join("testdata");
    fs::create_dir(&dir).expect("failed to create test dir");
    (parent, dir)
}

#[test]
fn test_discover_ignores_non_test_files() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("foo.txt"), "").expect("failed to write test file");
    fs::write(dir.join("bar.json"), "").expect("failed to write test file");

    let result = finder::discover_in(&dir, "foo").expect("failed to discover tests");
    assert!(result.found.is_empty());
}

#[test]
fn test_discover_ignores_hidden_directories() {
    let (_parent, dir) = test_dir();
    let hidden_dir = dir.join(".hidden");
    fs::create_dir(&hidden_dir).expect("failed to create test dir");
    fs::write(hidden_dir.join("secret.tdb"), "").expect("failed to write test file");
    fs::write(dir.join("visible.tdb"), "").expect("failed to write test file");

    let result = finder::discover_in(&dir, ".tdb").expect("failed to discover tests");
    assert_eq!(result.found.len(), 1);
    assert!(result.found[0].contains("visible"));
}

#[test]
fn test_discover_no_matches_returns_empty() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("foo.tdb"), "").expect("failed to write test file");

    let result = finder::discover_in(&dir, "nonexistent").expect("failed to discover tests");
    assert!(result.found.is_empty());
}

#[test]
fn test_discover_empty_directory() {
    let (_parent, dir) = test_dir();
    let result = finder::discover_in(&dir, "anything").expect("failed to discover tests");
    assert!(result.found.is_empty());
}

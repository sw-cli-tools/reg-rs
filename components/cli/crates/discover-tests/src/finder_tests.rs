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
fn test_discover_finds_matching_tdb_files() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("hello.tdb"), "").expect("failed to write test file");
    fs::write(dir.join("world.tdb"), "").expect("failed to write test file");
    fs::write(dir.join("other.txt"), "").expect("failed to write test file");

    let result = finder::discover_in(&dir, ".tdb").expect("failed to discover tests");
    assert_eq!(result.found.len(), 2);

    let result = finder::discover_in(&dir, "hello").expect("failed to discover tests");
    assert_eq!(result.found.len(), 1);
    assert!(result.found[0].ends_with("hello.tdb"));
}

#[test]
fn test_discover_finds_rgt_files() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("hello.rgt"), "").expect("failed to write test file");
    fs::write(dir.join("world.rgt"), "").expect("failed to write test file");

    let result = finder::discover_in(&dir, ".rgt").expect("failed to discover tests");
    assert_eq!(result.found.len(), 2);

    let result = finder::discover_in(&dir, "hello").expect("failed to discover tests");
    assert_eq!(result.found.len(), 1);
    assert!(result.found[0].ends_with("hello.rgt"));
}

#[test]
fn test_discover_rgt_takes_precedence_over_tdb() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("hello.rgt"), "").expect("failed to write test file");
    fs::write(dir.join("hello.tdb"), "").expect("failed to write test file");
    fs::write(dir.join("world.tdb"), "").expect("failed to write test file");

    let result = finder::discover_in(&dir, ".tdb").expect("failed to discover tests");
    assert_eq!(result.found.len(), 2);
    // hello should resolve to .rgt, world to .tdb
    assert!(result.found.iter().any(|p| p.ends_with("hello.rgt")));
    assert!(result.found.iter().any(|p| p.ends_with("world.tdb")));
}

#[test]
fn test_discover_tdb_pattern_matches_all() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("a.rgt"), "").expect("failed to write test file");
    fs::write(dir.join("b.tdb"), "").expect("failed to write test file");

    // ".tdb" pattern should match both .rgt and .tdb files
    let result = finder::discover_in(&dir, ".tdb").expect("failed to discover tests");
    assert_eq!(result.found.len(), 2);
}

#[test]
fn test_discover_matches_by_substring() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("test_a.tdb"), "").expect("failed to write test file");
    fs::write(dir.join("test_b.tdb"), "").expect("failed to write test file");
    fs::write(dir.join("other.tdb"), "").expect("failed to write test file");

    let result = finder::discover_in(&dir, "test_").expect("failed to discover tests");
    assert_eq!(result.found.len(), 2);
}

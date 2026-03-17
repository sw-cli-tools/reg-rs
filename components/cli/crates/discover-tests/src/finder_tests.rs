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
fn test_discover_finds_matching_tdb_files() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("hello.tdb"), "").unwrap();
    fs::write(dir.join("world.tdb"), "").unwrap();
    fs::write(dir.join("other.txt"), "").unwrap();

    let result = finder::discover_in(&dir, ".tdb").unwrap();
    assert_eq!(result.found.len(), 2);

    let result = finder::discover_in(&dir, "hello").unwrap();
    assert_eq!(result.found.len(), 1);
    assert!(result.found[0].ends_with("hello.tdb"));
}

#[test]
fn test_discover_finds_rgt_files() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("hello.rgt"), "").unwrap();
    fs::write(dir.join("world.rgt"), "").unwrap();

    let result = finder::discover_in(&dir, ".rgt").unwrap();
    assert_eq!(result.found.len(), 2);

    let result = finder::discover_in(&dir, "hello").unwrap();
    assert_eq!(result.found.len(), 1);
    assert!(result.found[0].ends_with("hello.rgt"));
}

#[test]
fn test_discover_rgt_takes_precedence_over_tdb() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("hello.rgt"), "").unwrap();
    fs::write(dir.join("hello.tdb"), "").unwrap();
    fs::write(dir.join("world.tdb"), "").unwrap();

    let result = finder::discover_in(&dir, ".tdb").unwrap();
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
    let result = finder::discover_in(&dir, ".tdb").unwrap();
    assert_eq!(result.found.len(), 2);
}

#[test]
fn test_discover_matches_by_substring() {
    let (_parent, dir) = test_dir();
    fs::write(dir.join("test_a.tdb"), "").unwrap();
    fs::write(dir.join("test_b.tdb"), "").unwrap();
    fs::write(dir.join("other.tdb"), "").unwrap();

    let result = finder::discover_in(&dir, "test_").unwrap();
    assert_eq!(result.found.len(), 2);
}

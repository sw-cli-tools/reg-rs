use std::path::PathBuf;

/// Return the test data directory (project-local, git-ignored).
pub fn test_data_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("work")
        .join("reg-rs");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn setup() {
    // Test data dir is passed to subprocesses via REG_RS_DATA_DIR in run_command()
}

/// Remove all files for a given test stem (.rgt, .out, .err, .tdb, locks).
pub fn cleanup_test_files(data_dir: &std::path::Path, stem: &str) {
    for ext in &["rgt", "out", "err", "tdb", "tdb.lock", "rgt.lock"] {
        let _ = std::fs::remove_file(data_dir.join(format!("{}.{}", stem, ext)));
    }
}

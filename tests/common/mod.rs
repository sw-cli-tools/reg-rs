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

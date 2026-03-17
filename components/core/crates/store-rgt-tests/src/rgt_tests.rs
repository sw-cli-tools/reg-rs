use reg_rs_store_rgt::rgt;
use reg_rs_store_rgt::rgt_util;
use tempfile::TempDir;

#[test]
fn test_parse_rgt_minimal() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("test.rgt");
    std::fs::write(&path, "command = \"echo hello\"\n").expect("failed to write test file");
    let spec = rgt::parse_rgt(path.to_str().expect("temp path is valid UTF-8"))
        .expect("failed to parse rgt");
    assert_eq!(spec.command, "echo hello");
    assert!(spec.timeout.is_none());
    assert!(spec.exit_code.is_none());
}

#[test]
fn test_parse_rgt_full() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("test.rgt");
    std::fs::write(
        &path,
        r#"
command = "echo hello"
timeout = 10
preprocess = "sed 's/x/y/g'"
diff_mode = "json"
exit_code = 0
desc = "A test"
expects = "hello"
flaky_note = "None"
"#,
    )
    .expect("failed to write test file");
    let spec = rgt::parse_rgt(path.to_str().expect("temp path is valid UTF-8"))
        .expect("failed to parse rgt");
    assert_eq!(spec.command, "echo hello");
    assert_eq!(spec.timeout, Some(10));
    assert_eq!(spec.exit_code, Some(0));
    assert_eq!(spec.desc.as_deref(), Some("A test"));
}

#[test]
fn test_write_rgt_roundtrip() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("test.rgt");
    let spec = rgt::RgtSpec {
        command: "echo hello".to_string(),
        timeout: Some(10),
        preprocess: None,
        diff_mode: None,
        exit_code: Some(0),
        desc: Some("A test".to_string()),
        expects: None,
        flaky_note: None,
    };
    rgt::write_rgt(path.to_str().expect("temp path is valid UTF-8"), &spec)
        .expect("failed to write rgt");
    let parsed = rgt::parse_rgt(path.to_str().expect("temp path is valid UTF-8"))
        .expect("failed to parse rgt");
    assert_eq!(parsed.command, "echo hello");
    assert_eq!(parsed.timeout, Some(10));
    assert_eq!(parsed.exit_code, Some(0));
}

#[test]
fn test_write_baseline_with_stderr() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let rgt = dir.path().join("test.rgt");
    rgt::write_baseline(
        rgt.to_str().expect("temp path is valid UTF-8"),
        "out\n",
        "err\n",
    )
    .expect("failed to write baseline");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("test.out")).expect("failed to read .out file"),
        "out\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("test.err")).expect("failed to read .err file"),
        "err\n"
    );
}

#[test]
fn test_tdb_path_for_rgt() {
    assert_eq!(rgt_util::tdb_path_for_rgt("/tmp/test.rgt"), "/tmp/test.tdb");
}

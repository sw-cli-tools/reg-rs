use reg_rs_store::queries;
use reg_rs_store::queries::statements;
use reg_rs_store::sqlite;
use reg_rs_store::sqlite_diff;
use tempfile::TempDir;

fn test_db() -> (TempDir, String) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("test.db");
    (
        dir,
        path.to_str().expect("temp path is valid UTF-8").to_string(),
    )
}

fn create_original_table(db: &str) {
    sqlite::create_table(
        db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
        ),
    )
    .expect("failed to create original table");
}

#[test]
fn test_write_and_read_differences() {
    let (_dir, db) = test_db();
    sqlite::create_table(
        &db,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::CREATE_DIFFERENCES_TABLE_TEMPLATE,
        ),
    )
    .expect("failed to create differences table");
    sqlite_diff::write_difference(&db, "1", "exit code 42").expect("failed to write difference");
    sqlite_diff::write_difference(&db, "3", "stderr output").expect("failed to write difference");
    let diffs = sqlite_diff::read_differences(
        &db,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::SELECT_DIFFERENCES_TEMPLATE,
        ),
    )
    .expect("failed to read differences");
    assert_eq!(diffs.len(), 2);
    assert_eq!(diffs[0], ("1".to_string(), "exit code 42".to_string()));
    assert_eq!(diffs[1], ("3".to_string(), "stderr output".to_string()));
}

#[test]
fn test_store_and_read_metadata() {
    let (_dir, db) = test_db();
    sqlite_diff::store_metadata(&db, "preprocess", "jq --sort-keys")
        .expect("failed to store metadata");
    let val = sqlite_diff::read_metadata(&db, "preprocess").expect("failed to read metadata");
    assert_eq!(val, Some("jq --sort-keys".to_string()));
}

#[test]
fn test_read_metadata_missing_key() {
    let (_dir, db) = test_db();
    sqlite_diff::store_metadata(&db, "preprocess", "cat").expect("failed to store metadata");
    let val = sqlite_diff::read_metadata(&db, "nonexistent").expect("failed to read metadata");
    assert_eq!(val, None);
}

#[test]
fn test_read_metadata_no_table() {
    let (_dir, db) = test_db();
    create_original_table(&db);
    let val = sqlite_diff::read_metadata(&db, "preprocess").expect("failed to read metadata");
    assert_eq!(val, None);
}

#[test]
fn test_store_metadata_upsert() {
    let (_dir, db) = test_db();
    sqlite_diff::store_metadata(&db, "preprocess", "cat").expect("failed to store metadata");
    sqlite_diff::store_metadata(&db, "preprocess", "sort").expect("failed to store metadata");
    let val = sqlite_diff::read_metadata(&db, "preprocess").expect("failed to read metadata");
    assert_eq!(val, Some("sort".to_string()));
}

use super::*;
use tempfile::TempDir;

fn test_db() -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.db");
    (dir, path.to_str().unwrap().to_string())
}

fn create_original_table(db: &str) {
    create_table(
        db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
        ),
    )
    .unwrap();
}

fn sample_results() -> TestResults {
    TestResults {
        name: "test1".to_string(),
        command: "echo hello".to_string(),
        time_created: "2024-01-01T12:00:00".to_string(),
        exit_code: 0,
        stderr: "".to_string(),
        stdout: "hello\n".to_string(),
    }
}

#[test]
fn test_create_and_drop_table() {
    let (_dir, db) = test_db();
    create_original_table(&db);
    drop_table(
        &db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::DROP_TABLE_TEMPLATE,
        ),
    )
    .unwrap();
}

#[test]
fn test_write_and_read_results() {
    let (_dir, db) = test_db();
    create_original_table(&db);
    write_results(
        &db,
        &sample_results(),
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::INSERT_TEST_RESULTS_TEMPLATE,
        ),
    )
    .unwrap();
    let read = read_results(
        &db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::SELECT_TEST_RESULTS_TEMPLATE,
        ),
    )
    .unwrap();
    assert_eq!(read.name, "test1");
    assert_eq!(read.command, "echo hello");
    assert_eq!(read.exit_code, 0);
    assert_eq!(read.stdout, "hello\n");
    assert_eq!(read.stderr, "");
}

#[test]
fn test_count_rows() {
    let (_dir, db) = test_db();
    create_original_table(&db);
    let count = count_rows(
        &db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )
    .unwrap();
    assert_eq!(count, 0);

    write_results(
        &db,
        &sample_results(),
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::INSERT_TEST_RESULTS_TEMPLATE,
        ),
    )
    .unwrap();
    let count = count_rows(
        &db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )
    .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_delete_all_rows() {
    let (_dir, db) = test_db();
    create_original_table(&db);
    write_results(
        &db,
        &sample_results(),
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::INSERT_TEST_RESULTS_TEMPLATE,
        ),
    )
    .unwrap();
    delete_all_rows(
        &db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::DELETE_ALL_ROWS_TEMPLATE,
        ),
    )
    .unwrap();
    let count = count_rows(
        &db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )
    .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_write_and_read_differences() {
    let (_dir, db) = test_db();
    create_table(
        &db,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::CREATE_DIFFERENCES_TABLE_TEMPLATE,
        ),
    )
    .unwrap();
    write_difference(&db, "1", "exit code 42").unwrap();
    write_difference(&db, "3", "stderr output").unwrap();
    let diffs = read_differences(
        &db,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::SELECT_DIFFERENCES_TEMPLATE,
        ),
    )
    .unwrap();
    assert_eq!(diffs.len(), 2);
    assert_eq!(diffs[0], ("1".to_string(), "exit code 42".to_string()));
    assert_eq!(diffs[1], ("3".to_string(), "stderr output".to_string()));
}

#[test]
fn test_count_differences_by_type() {
    let (_dir, db) = test_db();
    create_table(
        &db,
        &queries::get_statement(
            &queries::StatementContext::differences(),
            statements::CREATE_DIFFERENCES_TABLE_TEMPLATE,
        ),
    )
    .unwrap();
    write_difference(&db, "1", "code1").unwrap();
    write_difference(&db, "1", "code2").unwrap();
    write_difference(&db, "3", "stderr").unwrap();
    let count = super::count_differences_by_type(
        &db,
        &queries::get_statement(
            &queries::StatementContext::difference_count_by_type(1),
            statements::COUNT_DIFF_TYPE_TEMPLATE,
        ),
    )
    .unwrap();
    assert_eq!(count, 2);
    let count = super::count_differences_by_type(
        &db,
        &queries::get_statement(
            &queries::StatementContext::difference_count_by_type(3),
            statements::COUNT_DIFF_TYPE_TEMPLATE,
        ),
    )
    .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_read_results_empty_table_returns_error() {
    let (_dir, db) = test_db();
    create_original_table(&db);
    let result = read_results(
        &db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::SELECT_TEST_RESULTS_TEMPLATE,
        ),
    );
    assert!(result.is_err());
}

#[test]
fn test_store_and_read_metadata() {
    let (_dir, db) = test_db();
    store_metadata(&db, "preprocess", "jq --sort-keys").unwrap();
    let val = read_metadata(&db, "preprocess").unwrap();
    assert_eq!(val, Some("jq --sort-keys".to_string()));
}

#[test]
fn test_read_metadata_missing_key() {
    let (_dir, db) = test_db();
    store_metadata(&db, "preprocess", "cat").unwrap();
    let val = read_metadata(&db, "nonexistent").unwrap();
    assert_eq!(val, None);
}

#[test]
fn test_read_metadata_no_table() {
    let (_dir, db) = test_db();
    // Create a DB file without metadata table
    create_original_table(&db);
    let val = read_metadata(&db, "preprocess").unwrap();
    assert_eq!(val, None);
}

#[test]
fn test_store_metadata_upsert() {
    let (_dir, db) = test_db();
    store_metadata(&db, "preprocess", "cat").unwrap();
    store_metadata(&db, "preprocess", "sort").unwrap();
    let val = read_metadata(&db, "preprocess").unwrap();
    assert_eq!(val, Some("sort".to_string()));
}

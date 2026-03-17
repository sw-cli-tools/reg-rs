use reg_rs_store::queries;
use reg_rs_store::queries::statements;
use reg_rs_store::sqlite;
use reg_rs_store::sqlite_diff;
use reg_rs_types::types::TestResults;
use tempfile::TempDir;

fn test_db() -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.db");
    (dir, path.to_str().unwrap().to_string())
}

fn create_original_table(db: &str) {
    sqlite::create_table(
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
    sqlite::drop_table(
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
    sqlite::write_results(
        &db,
        &sample_results(),
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::INSERT_TEST_RESULTS_TEMPLATE,
        ),
    )
    .unwrap();
    let read = sqlite::read_results(
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
    let count = sqlite_diff::count_rows(
        &db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )
    .unwrap();
    assert_eq!(count, 0);

    sqlite::write_results(
        &db,
        &sample_results(),
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::INSERT_TEST_RESULTS_TEMPLATE,
        ),
    )
    .unwrap();
    let count = sqlite_diff::count_rows(
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
    sqlite::write_results(
        &db,
        &sample_results(),
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::INSERT_TEST_RESULTS_TEMPLATE,
        ),
    )
    .unwrap();
    sqlite::delete_all_rows(
        &db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::DELETE_ALL_ROWS_TEMPLATE,
        ),
    )
    .unwrap();
    let count = sqlite_diff::count_rows(
        &db,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )
    .unwrap();
    assert_eq!(count, 0);
}

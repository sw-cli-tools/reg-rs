use reg_rs_store::db;
use reg_rs_store::queries;
use reg_rs_store::queries::statements;
use reg_rs_store::sqlite;
use reg_rs_store::sqlite_diff;
use reg_rs_types::types::TestResults;
use tempfile::TempDir;

fn create_test_results(name: &str) -> TestResults {
    TestResults {
        name: name.to_string(),
        command: "echo hello".to_string(),
        time_created: "2024-01-01 12:00:00".to_string(),
        exit_code: 0,
        stderr: "".to_string(),
        stdout: "hello\n".to_string(),
    }
}

fn setup_latest_with_results(db_path: &str, results: &TestResults) {
    let ctx = queries::StatementContext::latest();
    sqlite::create_table(
        db_path,
        &queries::get_statement(&ctx, statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE),
    )
    .expect("failed to create latest table");
    sqlite::write_results(
        db_path,
        results,
        &queries::get_statement(&ctx, statements::INSERT_TEST_RESULTS_TEMPLATE),
    )
    .expect("failed to write latest results");
}

fn count_latest_rows(db_path: &str) -> u32 {
    sqlite_diff::count_rows(
        db_path,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            statements::COUNT_TABLE_ROWS_TEMPLATE,
        ),
    )
    .expect("failed to count latest rows")
}

#[test]
fn test_sqlite_store_and_read_results() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let db_path = temp_dir.path().join("test1.db");
    let db_path_str = db_path.to_str().expect("temp path is valid UTF-8");

    let test_results = create_test_results("test1");

    sqlite::create_table(
        db_path_str,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
        ),
    )
    .expect("failed to create table");

    sqlite::write_results(
        db_path_str,
        &test_results,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::INSERT_TEST_RESULTS_TEMPLATE,
        ),
    )
    .expect("failed to write results");

    let read_results = sqlite::read_results(
        db_path_str,
        &queries::get_statement(
            &queries::StatementContext::original(),
            statements::SELECT_TEST_RESULTS_TEMPLATE,
        ),
    )
    .expect("failed to read results");

    assert_eq!(read_results.name, "test1");
    assert_eq!(read_results.command, "echo hello");
    assert_eq!(read_results.exit_code, 0);
    assert_eq!(read_results.stdout, "hello\n");
}

#[test]
fn test_replace_latest_results() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let db_path = temp_dir.path().join("test2.db");
    let db_path_str = db_path.to_str().expect("temp path is valid UTF-8");

    let test_results = create_test_results("test2");
    setup_latest_with_results(db_path_str, &test_results);
    assert_eq!(count_latest_rows(db_path_str), 1);

    let updated = create_test_results("test2_updated");
    db::replace_latest_results(db_path_str, &updated).expect("failed to replace latest results");
    assert_eq!(count_latest_rows(db_path_str), 1);

    let read = db::read_latest_results(db_path_str).expect("failed to read latest results");
    assert_eq!(read.name, "test2_updated");
}

#[test]
fn test_sqlite_count_and_clear() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let db_path = temp_dir.path().join("test3.db");
    let db_path_str = db_path.to_str().expect("temp path is valid UTF-8");

    let test_results = create_test_results("test3");
    setup_latest_with_results(db_path_str, &test_results);

    assert_eq!(count_latest_rows(db_path_str), 1);

    sqlite::delete_all_rows(
        db_path_str,
        &queries::get_statement(
            &queries::StatementContext::latest(),
            statements::DELETE_ALL_ROWS_TEMPLATE,
        ),
    )
    .expect("failed to delete all rows");

    assert_eq!(count_latest_rows(db_path_str), 0);
}

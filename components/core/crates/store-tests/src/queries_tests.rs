use reg_rs_store::queries;
use reg_rs_store::queries::statements;

#[test]
fn test_get_statement_original_table() {
    let stmt = queries::get_statement(
        &queries::StatementContext::original(),
        statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
    );
    assert!(stmt.contains("original_results_table"));
}

#[test]
fn test_get_statement_latest_table() {
    let stmt = queries::get_statement(
        &queries::StatementContext::latest(),
        statements::CREATE_TEST_RESULTS_TABLE_TEMPLATE,
    );
    assert!(stmt.contains("latest_results_table"));
}

#[test]
fn test_get_statement_differences_table() {
    let stmt = queries::get_statement(
        &queries::StatementContext::differences(),
        statements::CREATE_DIFFERENCES_TABLE_TEMPLATE,
    );
    assert!(stmt.contains("differences_table"));
}

#[test]
fn test_get_statement_drop_table() {
    let stmt = queries::get_statement(
        &queries::StatementContext::original(),
        statements::DROP_TABLE_TEMPLATE,
    );
    assert!(stmt.contains("DROP TABLE IF EXISTS"));
    assert!(stmt.contains("original_results_table"));
}

#[test]
fn test_get_statement_count_diff_type() {
    let stmt = queries::get_statement(
        &queries::StatementContext::difference_count_by_type(3),
        statements::COUNT_DIFF_TYPE_TEMPLATE,
    );
    assert!(stmt.contains("differences_table"));
    assert!(stmt.contains("3"));
}

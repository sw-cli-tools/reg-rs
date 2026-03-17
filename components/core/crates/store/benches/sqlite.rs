use criterion::{Criterion, criterion_group, criterion_main};
use reg_rs_store::db;
use reg_rs_store::db_ops;
use reg_rs_store::queries::StatementContext;
use reg_rs_types::types::{RegressionType, TestResults};
use tempfile::TempDir;

fn make_test_results() -> TestResults {
    TestResults {
        name: "bench_test".to_string(),
        command: "echo hello".to_string(),
        time_created: "2024-01-01T00:00:00Z".to_string(),
        exit_code: 0,
        stderr: String::new(),
        stdout: "hello\n".to_string(),
    }
}

fn setup_db_with_results(dir: &TempDir) -> String {
    let db_path = dir.path().join("bench.tdb").to_string_lossy().to_string();
    let results = make_test_results();
    db::store_results(&db_path, &results, StatementContext::original()).unwrap();
    db_path
}

fn setup_db_with_differences(dir: &TempDir) -> String {
    let db_path = setup_db_with_results(dir);
    db_ops::reset_differences(&db_path).unwrap();
    db_ops::store_difference(&db_path, RegressionType::StdoutAdd, "added line 1").unwrap();
    db_ops::store_difference(&db_path, RegressionType::StdoutRemove, "removed line 1").unwrap();
    db_ops::store_difference(&db_path, RegressionType::StderrAdd, "stderr added").unwrap();
    db_path
}

fn sqlite_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("sqlite");
    group.sample_size(50);

    group.bench_function("store_results", |b| {
        b.iter_with_setup(
            || {
                let dir = TempDir::new().unwrap();
                let db_path = dir.path().join("bench.tdb").to_string_lossy().to_string();
                (dir, db_path)
            },
            |(_dir, db_path)| {
                let results = make_test_results();
                db::store_results(&db_path, &results, StatementContext::original()).unwrap();
            },
        )
    });

    group.bench_function("read_results", |b| {
        b.iter_with_setup(
            || {
                let dir = TempDir::new().unwrap();
                let db_path = setup_db_with_results(&dir);
                (dir, db_path)
            },
            |(_dir, db_path)| {
                db::read_original_results(&db_path).unwrap();
            },
        )
    });

    group.bench_function("store_and_read_metadata", |b| {
        b.iter_with_setup(
            || {
                let dir = TempDir::new().unwrap();
                let db_path = setup_db_with_results(&dir);
                (dir, db_path)
            },
            |(_dir, db_path)| {
                db_ops::store_metadata(&db_path, "bench_key", "bench_value").unwrap();
                db_ops::read_metadata(&db_path, "bench_key").unwrap();
            },
        )
    });

    group.bench_function("count_differences", |b| {
        b.iter_with_setup(
            || {
                let dir = TempDir::new().unwrap();
                let db_path = setup_db_with_differences(&dir);
                (dir, db_path)
            },
            |(_dir, db_path)| {
                db_ops::count_differences(&db_path).unwrap();
            },
        )
    });

    group.finish();
}

criterion_group!(benches, sqlite_benchmarks);
criterion_main!(benches);

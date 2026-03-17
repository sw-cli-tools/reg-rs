use criterion::{Criterion, criterion_group, criterion_main};
use reg_rs_store::db;
use reg_rs_store::queries;
use reg_rs_store_rgt::rgt::{self, RgtSpec};
use reg_rs_types::normalize::{DiffMode, apply};
use reg_rs_types::types::TestResults;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Group 1: normalize (pure CPU, no I/O)
// ---------------------------------------------------------------------------

fn bench_normalize_text(c: &mut Criterion) {
    let input = "hello world\nfoo bar\nbaz qux\n".repeat(100);
    c.bench_function("normalize_text_100_lines", |b| {
        b.iter(|| apply(&input, &DiffMode::Text))
    });
}

fn bench_normalize_json(c: &mut Criterion) {
    let input = r#"{"z": 1, "a": 2, "m": {"z": 3, "a": 4}}"#;
    c.bench_function("normalize_json_small", |b| {
        b.iter(|| apply(input, &DiffMode::Json))
    });
}

fn bench_normalize_lines_unordered(c: &mut Criterion) {
    let input = (0..100)
        .rev()
        .map(|i| format!("line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    c.bench_function("normalize_lines_unordered_100", |b| {
        b.iter(|| apply(&input, &DiffMode::LinesUnordered))
    });
}

// ---------------------------------------------------------------------------
// Group 2: rgt parse/write (file I/O)
// ---------------------------------------------------------------------------

fn bench_rgt_parse(c: &mut Criterion) {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("bench.rgt");
    let content = "\
command = \"echo hello\"\n\
timeout = 10\n\
preprocess = \"cat\"\n\
diff_mode = \"json\"\n\
exit_code = 0\n\
desc = \"A benchmark test\"\n";
    std::fs::write(&path, content).expect("write rgt");
    let path_str = path.to_str().expect("path utf8");
    c.bench_function("rgt_parse", |b| b.iter(|| rgt::parse_rgt(path_str)));
}

fn bench_rgt_write(c: &mut Criterion) {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("bench_write.rgt");
    let path_str = path.to_str().expect("path utf8").to_string();
    let spec = RgtSpec {
        command: "echo hello".to_string(),
        timeout: Some(10),
        preprocess: Some("cat".to_string()),
        diff_mode: Some("json".to_string()),
        exit_code: Some(0),
        desc: Some("A benchmark test".to_string()),
        expects: None,
        flaky_note: None,
    };
    c.bench_function("rgt_write", |b| b.iter(|| rgt::write_rgt(&path_str, &spec)));
}

// ---------------------------------------------------------------------------
// Group 3: sqlite store roundtrip (file I/O + SQLite)
// ---------------------------------------------------------------------------

fn bench_store_and_read_results(c: &mut Criterion) {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("bench.tdb");
    let db_str = db_path.to_str().expect("path utf8").to_string();
    let results = TestResults {
        name: "bench_test".to_string(),
        command: "echo hello".to_string(),
        time_created: "2026-01-01T00:00:00".to_string(),
        exit_code: 0,
        stdout: "hello\n".to_string(),
        stderr: "".to_string(),
    };
    // Store initial results so reads work
    db::store_results(&db_str, &results, queries::StatementContext::original()).expect("store");

    c.bench_function("sqlite_store_results", |b| {
        b.iter(|| db::store_results(&db_str, &results, queries::StatementContext::original()))
    });

    c.bench_function("sqlite_read_results", |b| {
        b.iter(|| db::read_original_results(&db_str))
    });
}

// ---------------------------------------------------------------------------
// Combine all groups
// ---------------------------------------------------------------------------

criterion_group!(
    normalize,
    bench_normalize_text,
    bench_normalize_json,
    bench_normalize_lines_unordered,
);

criterion_group!(rgt_io, bench_rgt_parse, bench_rgt_write,);

criterion_group!(sqlite_io, bench_store_and_read_results,);

criterion_main!(normalize, rgt_io, sqlite_io);

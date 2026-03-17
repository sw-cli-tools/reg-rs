use criterion::{Criterion, criterion_group, criterion_main};
use reg_rs_store_rgt::rgt::{self, RgtSpec};
use tempfile::TempDir;

fn make_minimal_spec() -> RgtSpec {
    RgtSpec {
        command: "echo hello".to_string(),
        timeout: None,
        preprocess: None,
        diff_mode: None,
        exit_code: None,
        desc: None,
        expects: None,
        flaky_note: None,
    }
}

fn make_full_spec() -> RgtSpec {
    RgtSpec {
        command: "curl -s https://api.example.com/health | jq .status".to_string(),
        timeout: Some(30),
        preprocess: Some("sed 's/[0-9]\\{10\\}/TIMESTAMP/g'".to_string()),
        diff_mode: Some("json".to_string()),
        exit_code: Some(0),
        desc: Some("Health check endpoint returns valid JSON status".to_string()),
        expects: Some("Should return {\"status\": \"ok\"} within 30 seconds".to_string()),
        flaky_note: Some("May timeout under heavy load; retry once".to_string()),
    }
}

fn rgt_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("rgt");
    group.sample_size(50);

    group.bench_function("rgt_parse_minimal", |b| {
        b.iter_with_setup(
            || {
                let dir = TempDir::new().unwrap();
                let path = dir.path().join("test.rgt").to_string_lossy().to_string();
                rgt::write_rgt(&path, &make_minimal_spec()).unwrap();
                (dir, path)
            },
            |(_dir, path)| {
                rgt::parse_rgt(&path).unwrap();
            },
        )
    });

    group.bench_function("rgt_parse_full", |b| {
        b.iter_with_setup(
            || {
                let dir = TempDir::new().unwrap();
                let path = dir.path().join("test.rgt").to_string_lossy().to_string();
                rgt::write_rgt(&path, &make_full_spec()).unwrap();
                (dir, path)
            },
            |(_dir, path)| {
                rgt::parse_rgt(&path).unwrap();
            },
        )
    });

    group.bench_function("rgt_write", |b| {
        b.iter_with_setup(
            || {
                let dir = TempDir::new().unwrap();
                let path = dir.path().join("test.rgt").to_string_lossy().to_string();
                (dir, path)
            },
            |(_dir, path)| {
                rgt::write_rgt(&path, &make_full_spec()).unwrap();
            },
        )
    });

    group.bench_function("rgt_write_baseline", |b| {
        b.iter_with_setup(
            || {
                let dir = TempDir::new().unwrap();
                let path = dir.path().join("test.rgt").to_string_lossy().to_string();
                rgt::write_rgt(&path, &make_minimal_spec()).unwrap();
                (dir, path)
            },
            |(_dir, path)| {
                rgt::write_baseline(
                    &path,
                    "Expected stdout output line 1\nExpected stdout output line 2\n",
                    "Some stderr warning\n",
                )
                .unwrap();
            },
        )
    });

    group.bench_function("rgt_roundtrip", |b| {
        b.iter_with_setup(
            || {
                let dir = TempDir::new().unwrap();
                let path = dir.path().join("test.rgt").to_string_lossy().to_string();
                (dir, path)
            },
            |(_dir, path)| {
                rgt::write_rgt(&path, &make_full_spec()).unwrap();
                rgt::parse_rgt(&path).unwrap();
            },
        )
    });

    group.finish();
}

criterion_group!(benches, rgt_benchmarks);
criterion_main!(benches);

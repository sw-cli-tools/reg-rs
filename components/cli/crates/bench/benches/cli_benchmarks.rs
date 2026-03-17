use criterion::{Criterion, criterion_group, criterion_main};
use reg_rs_discover::finder;
use std::fs;

fn bench_discover_10_files(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("temp dir");
    let sub = dir.path().join("tests");
    fs::create_dir(&sub).expect("create dir");
    for i in 0..10 {
        fs::write(sub.join(format!("test_{}.rgt", i)), "").expect("write file");
    }
    c.bench_function("discover_10_rgt_files", |b| {
        b.iter(|| finder::discover_in(&sub, ".rgt"))
    });
}

fn bench_discover_100_files(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("temp dir");
    let sub = dir.path().join("tests");
    fs::create_dir(&sub).expect("create dir");
    for i in 0..100 {
        fs::write(sub.join(format!("test_{}.rgt", i)), "").expect("write file");
    }
    c.bench_function("discover_100_rgt_files", |b| {
        b.iter(|| finder::discover_in(&sub, ".rgt"))
    });
}

fn bench_discover_with_dedup(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("temp dir");
    let sub = dir.path().join("tests");
    fs::create_dir(&sub).expect("create dir");
    // Create both .rgt and .tdb files — discovery must deduplicate
    for i in 0..50 {
        fs::write(sub.join(format!("test_{}.rgt", i)), "").expect("write rgt");
        fs::write(sub.join(format!("test_{}.tdb", i)), "").expect("write tdb");
    }
    c.bench_function("discover_50_with_dedup", |b| {
        b.iter(|| finder::discover_in(&sub, ".tdb"))
    });
}

fn bench_discover_pattern_filter(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("temp dir");
    let sub = dir.path().join("tests");
    fs::create_dir(&sub).expect("create dir");
    for i in 0..100 {
        let prefix = if i % 10 == 0 { "match" } else { "other" };
        fs::write(sub.join(format!("{}_{}.rgt", prefix, i)), "").expect("write file");
    }
    c.bench_function("discover_pattern_filter_100", |b| {
        b.iter(|| finder::discover_in(&sub, "match"))
    });
}

criterion_group!(
    benches,
    bench_discover_10_files,
    bench_discover_100_files,
    bench_discover_with_dedup,
    bench_discover_pattern_filter,
);
criterion_main!(benches);

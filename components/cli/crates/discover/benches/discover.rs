use std::fs;
use std::path::Path;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tempfile::TempDir;

fn create_rgt_files(dir: &Path, count: usize) {
    for i in 0..count {
        let path = dir.join(format!("test_{:04}.rgt", i));
        fs::write(&path, format!("# test {}", i)).unwrap();
    }
}

fn create_tdb_files(dir: &Path, count: usize) {
    for i in 0..count {
        let path = dir.join(format!("test_{:04}.tdb", i));
        fs::write(&path, "").unwrap();
    }
}

fn bench_discover_rgt(c: &mut Criterion) {
    let mut group = c.benchmark_group("discover_rgt");
    group.sample_size(20);

    for size in [10, 50, 100, 200] {
        let tmp = TempDir::new().unwrap();
        create_rgt_files(tmp.path(), size);

        group.bench_function(format!("{}_files", size), |b| {
            b.iter(|| black_box(reg_rs_discover::finder::discover_in(tmp.path(), ".rgt").unwrap()))
        });
    }
    group.finish();
}

fn bench_discover_dedup(c: &mut Criterion) {
    let mut group = c.benchmark_group("discover_dedup");
    group.sample_size(20);

    for size in [10, 50, 100, 200] {
        let tmp = TempDir::new().unwrap();
        // Create .rgt files for all, .tdb files for the first half (dedup needed)
        create_rgt_files(tmp.path(), size);
        create_tdb_files(tmp.path(), size / 2);

        group.bench_function(format!("{}_files_half_dup", size), |b| {
            b.iter(|| black_box(reg_rs_discover::finder::discover_in(tmp.path(), ".rgt").unwrap()))
        });
    }
    group.finish();
}

fn bench_discover_pattern_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("discover_pattern_filter");
    group.sample_size(20);

    let tmp = TempDir::new().unwrap();
    // Create 100 files; 10 will match the pattern "match"
    for i in 0..100 {
        let name = if i % 10 == 0 {
            format!("match_{:04}.rgt", i)
        } else {
            format!("test_{:04}.rgt", i)
        };
        fs::write(tmp.path().join(&name), format!("# {}", i)).unwrap();
    }

    group.bench_function("100_files_10pct_match", |b| {
        b.iter(|| black_box(reg_rs_discover::finder::discover_in(tmp.path(), "match").unwrap()))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_discover_rgt,
    bench_discover_dedup,
    bench_discover_pattern_filter,
);
criterion_main!(benches);

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};

fn make_lines(n: usize) -> String {
    (0..n)
        .map(|i| format!("line {} content here", i))
        .collect::<Vec<_>>()
        .join("\n")
}

fn make_lines_with_change(n: usize, changed_line: usize) -> String {
    (0..n)
        .map(|i| {
            if i == changed_line {
                format!("CHANGED line {} different content", i)
            } else {
                format!("line {} content here", i)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn bench_diff_identical(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_identical");
    for size in [100, 500, 1000] {
        let input = make_lines(size);
        let bytes = input.len() as u64;
        group.throughput(Throughput::Bytes(bytes));
        group.bench_function(format!("{}_lines", size), |b| {
            b.iter(|| black_box(reg_rs_runner::diff::get_differences(&input, &input)))
        });
    }
    group.finish();
}

fn bench_diff_small_change(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_small_change");
    for size in [100, 500, 1000] {
        let older = make_lines(size);
        let newer = make_lines_with_change(size, size / 2);
        let bytes = older.len() as u64;
        group.throughput(Throughput::Bytes(bytes));
        group.bench_function(format!("{}_lines", size), |b| {
            b.iter(|| black_box(reg_rs_runner::diff::get_differences(&older, &newer)))
        });
    }
    group.finish();
}

fn bench_diff_completely_different(c: &mut Criterion) {
    let older = make_lines(100);
    let newer = (0..100)
        .map(|i| format!("completely different line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let bytes = older.len() as u64;

    c.bench_function("diff_completely_different_100", |b| {
        b.iter(|| black_box(reg_rs_runner::diff::get_differences(&older, &newer)))
    });

    // Set throughput via a group wrapper
    let mut group = c.benchmark_group("diff_completely_different_throughput");
    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("100_lines", |b| {
        b.iter(|| black_box(reg_rs_runner::diff::get_differences(&older, &newer)))
    });
    group.finish();
}

fn bench_diff_empty_to_content(c: &mut Criterion) {
    let older = String::new();
    let newer = make_lines(100);
    let bytes = newer.len() as u64;

    c.bench_function("diff_empty_to_content", |b| {
        b.iter(|| black_box(reg_rs_runner::diff::get_differences(&older, &newer)))
    });

    let mut group = c.benchmark_group("diff_empty_to_content_throughput");
    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("100_lines", |b| {
        b.iter(|| black_box(reg_rs_runner::diff::get_differences(&older, &newer)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_diff_identical,
    bench_diff_small_change,
    bench_diff_completely_different,
    bench_diff_empty_to_content,
);
criterion_main!(benches);

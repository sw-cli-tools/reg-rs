use criterion::{Criterion, criterion_group, criterion_main};
use reg_rs_runner::diff;

fn bench_diff_identical(c: &mut Criterion) {
    let text = "line 1\nline 2\nline 3\n".repeat(100);
    c.bench_function("diff_identical_300_lines", |b| {
        b.iter(|| diff::get_differences(&text, &text))
    });
}

fn bench_diff_small_change(c: &mut Criterion) {
    let older = "line 1\nline 2\nline 3\n".repeat(100);
    let newer = older.replace("line 2", "line 2 modified");
    c.bench_function("diff_small_change_300_lines", |b| {
        b.iter(|| diff::get_differences(&older, &newer))
    });
}

fn bench_diff_completely_different(c: &mut Criterion) {
    let older = (0..100)
        .map(|i| format!("old line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let newer = (0..100)
        .map(|i| format!("new line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    c.bench_function("diff_completely_different_100_lines", |b| {
        b.iter(|| diff::get_differences(&older, &newer))
    });
}

fn bench_diff_empty_to_content(c: &mut Criterion) {
    let newer = "content\n".repeat(50);
    c.bench_function("diff_empty_to_50_lines", |b| {
        b.iter(|| diff::get_differences("", &newer))
    });
}

criterion_group!(
    benches,
    bench_diff_identical,
    bench_diff_small_change,
    bench_diff_completely_different,
    bench_diff_empty_to_content,
);
criterion_main!(benches);

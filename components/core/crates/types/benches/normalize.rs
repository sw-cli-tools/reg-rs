use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use reg_rs_types::normalize::{DiffMode, apply};

fn make_text_lines(n: usize) -> String {
    (0..n)
        .map(|i| format!("Line {}: some sample text content here", i))
        .collect::<Vec<_>>()
        .join("\n")
}

fn make_small_json() -> String {
    r#"{"name":"Alice","age":30,"address":{"street":"123 Main St","city":"Springfield"},"tags":["a","b","c"]}"#.to_string()
}

fn make_large_json(keys: usize) -> String {
    let entries: Vec<String> = (0..keys)
        .map(|i| format!(r#""key_{:04}": "value_{}""#, i, i))
        .collect();
    format!("{{{}}}", entries.join(","))
}

fn make_reversed_lines(n: usize) -> String {
    (0..n)
        .rev()
        .map(|i| format!("Line {:06}", i))
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_benchmarks(c: &mut Criterion) {
    // Text normalization
    {
        let mut group = c.benchmark_group("normalize_text");
        let input = make_text_lines(100);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_function("normalize_text_100_lines", |b| {
            b.iter(|| black_box(apply(black_box(&input), &DiffMode::Text).unwrap()))
        });
        group.finish();
    }

    // JSON normalization - small
    {
        let mut group = c.benchmark_group("normalize_json");
        let small_json = make_small_json();
        group.throughput(Throughput::Bytes(small_json.len() as u64));
        group.bench_function("normalize_json_small", |b| {
            b.iter(|| black_box(apply(black_box(&small_json), &DiffMode::Json).unwrap()))
        });

        let large_json = make_large_json(50);
        group.throughput(Throughput::Bytes(large_json.len() as u64));
        group.bench_function("normalize_json_large", |b| {
            b.iter(|| black_box(apply(black_box(&large_json), &DiffMode::Json).unwrap()))
        });
        group.finish();
    }

    // Lines unordered normalization
    {
        let mut group = c.benchmark_group("normalize_lines_unordered");
        let input_100 = make_reversed_lines(100);
        group.throughput(Throughput::Bytes(input_100.len() as u64));
        group.bench_function("normalize_lines_unordered_100", |b| {
            b.iter(|| black_box(apply(black_box(&input_100), &DiffMode::LinesUnordered).unwrap()))
        });

        let input_1000 = make_reversed_lines(1000);
        group.throughput(Throughput::Bytes(input_1000.len() as u64));
        group.bench_function("normalize_lines_unordered_1000", |b| {
            b.iter(|| black_box(apply(black_box(&input_1000), &DiffMode::LinesUnordered).unwrap()))
        });
        group.finish();
    }
}

criterion_group!(benches, normalize_benchmarks);
criterion_main!(benches);

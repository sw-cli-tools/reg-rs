# Benchmark Results

Captured 2026-03-17 using [criterion](https://crates.io/crates/criterion) (`cargo bench`).

**Machine:** Apple M3 Pro, 18 GB RAM, macOS (arm64)

## How to run

```bash
# All benchmarks for a component
cd components/core && cargo bench
cd components/engine && cargo bench
cd components/cli && cargo bench

# Filter by name
cd components/core && cargo bench -- normalize
cd components/engine && cargo bench -- diff_identical
```

HTML reports are generated in each component's `target/criterion/` directory.

## core — Normalization

| Benchmark | Median | Range |
|-----------|--------|-------|
| normalize_text_100_lines | 41 ns | 41–41 ns |
| normalize_json_small | 390 ns | 389–391 ns |
| normalize_json_large (50 keys) | 2.35 µs | 2.35–2.36 µs |
| normalize_lines_unordered_100 | 2.04 µs | 2.03–2.04 µs |
| normalize_lines_unordered_1000 | 18.2 µs | 18.1–18.4 µs |

## core — SQLite Store

| Benchmark | Median | Range |
|-----------|--------|-------|
| sqlite/store_results | 752 µs | 740–766 µs |
| sqlite/read_results | 172 µs | 170–175 µs |
| sqlite/store_and_read_metadata | 530 µs | 509–558 µs |
| sqlite/count_differences | 178 µs | 174–183 µs |

## core — .rgt Format

| Benchmark | Median | Range |
|-----------|--------|-------|
| rgt/rgt_parse_minimal | 90 µs | 89–92 µs |
| rgt/rgt_parse_full | 87 µs | 87–88 µs |
| rgt/rgt_write | 136 µs | 135–137 µs |
| rgt/rgt_write_baseline | 243 µs | 240–245 µs |
| rgt/rgt_roundtrip | 156 µs | 152–161 µs |

## engine — Diff

| Benchmark | Median | Range |
|-----------|--------|-------|
| diff_identical/100_lines | 10.5 µs | 10.5–10.5 µs |
| diff_identical/500_lines | 48.7 µs | 48.4–49.2 µs |
| diff_identical/1000_lines | 97.3 µs | 97.2–97.5 µs |
| diff_small_change/100_lines | 11.0 µs | 11.0–11.0 µs |
| diff_small_change/500_lines | 50.0 µs | 49.9–50.1 µs |
| diff_small_change/1000_lines | 100 µs | 100–100 µs |
| diff_completely_different_100 | 321 µs | 320–323 µs |
| diff_empty_to_content | 100 µs | 94–106 µs |

## cli — Test Discovery

| Benchmark | Median | Range |
|-----------|--------|-------|
| discover_rgt/10_files | 16.1 µs | 16.0–16.2 µs |
| discover_rgt/50_files | 16.1 µs | 16.1–16.1 µs |
| discover_rgt/100_files | 16.1 µs | 16.0–16.1 µs |
| discover_rgt/200_files | 15.9 µs | 15.6–16.3 µs |
| discover_dedup/50_files | 91.4 µs | 91.3–91.7 µs |
| discover_pattern_filter_100 | 90.9 µs | 89.6–92.2 µs |

## Key Observations

- **Normalization** is fast: text passthrough is 41ns, JSON sorting is 390ns even with nested keys
- **SQLite** operations dominate latency: store is ~750µs (file I/O + locking), read is ~172µs
- **Diff** scales linearly with input size: ~100ns/line for identical, ~1µs/line for different
- **Discovery** has a flat ~16µs base cost (directory walk overhead); dedup adds ~75µs for HashMap processing
- **rgt parse** (~90µs) is cheaper than SQLite read (~172µs), validating the .rgt-first architecture

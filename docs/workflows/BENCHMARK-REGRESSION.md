# Benchmark Regression Detection

## Overview

Automated performance benchmarking with regression detection using [Criterion.rs](https://github.com/bheisler/criterion.rs).

**Workflow:** `.github/workflows/benchmark-regression.yml`
**Tool:** Criterion.rs
**Triggers:** PR, Push to main, Manual dispatch
**Goal:** Prevent performance regressions

## How It Works

1. **Baseline**: Store performance metrics from main branch
2. **Current**: Run benchmarks on PR/current branch
3. **Compare**: Calculate performance change vs baseline
4. **Report**: Comment on PR with results
5. **Update**: Save new baseline when merging to main

## Setup Benchmarks

### Directory Structure

```
benches/
├── my_benchmark.rs
└── another_benchmark.rs
```

### Example Benchmark

**`benches/performance.rs`:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_template::expensive_function;

fn benchmark_expensive_function(c: &mut Criterion) {
    c.bench_function("expensive_function", |b| {
        b.iter(|| expensive_function(black_box(100)))
    });
}

fn benchmark_with_setup(c: &mut Criterion) {
    c.bench_function("with_setup", |b| {
        let data = vec![1, 2, 3, 4, 5];
        b.iter(|| {
            process(black_box(&data))
        })
    });
}

criterion_group!(benches, benchmark_expensive_function, benchmark_with_setup);
criterion_main!(benches);
```

**Add to `Cargo.toml`:**

```toml
[[bench]]
name = "performance"
harness = false
```

## Running Benchmarks

### Locally

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench performance

# Save baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

### CI Integration

The workflow automatically:
- Downloads baseline from main branch
- Runs current benchmarks
- Compares performance
- Posts results to PR
- Updates baseline on merge to main

## Understanding Results

### Benchmark Output

```
expensive_function      time:   [48.123 µs 48.567 µs 49.012 µs]
                        change: [-5.2341% -3.1234% -1.0123%] (p = 0.02 < 0.05)
                        Performance has improved.
```

**Interpretation:**
- **time**: Current execution time (min, median, max)
- **change**: % change from baseline
- **p-value**: Statistical significance (< 0.05 = significant)

### Performance Change

| Change | Interpretation | Action |
|--------|---------------|--------|
| < -5% | **Improvement** ✅ | Great! Document what improved |
| -5% to +5% | **No change** ⚪ | Within noise threshold |
| +5% to +20% | **Minor regression** ⚠️ | Investigate if acceptable |
| > +20% | **Major regression** ❌ | Must fix before merge |

### Statistical Significance

```
change: [+2.5% +5.2% +7.8%] (p = 0.45 > 0.05)
Change within noise threshold.
```

**Not significant** - variation likely due to noise, not real change.

```
change: [+15.2% +18.5% +21.3%] (p = 0.001 < 0.05)
Performance has regressed.
```

**Significant** - real performance degradation detected.

## Benchmark Best Practices

### Use `black_box`

```rust
// ❌ Bad - Compiler optimizes away
b.iter(|| expensive_function(100));

// ✅ Good - Prevents optimization
b.iter(|| expensive_function(black_box(100)));
```

### Avoid Setup in Measurement

```rust
// ❌ Bad - Includes allocation in measurement
b.iter(|| {
    let data = vec![1, 2, 3];  // Measured
    process(&data)
});

// ✅ Good - Setup outside measurement
let data = vec![1, 2, 3];
b.iter(|| process(black_box(&data)));
```

### Benchmark Representative Sizes

```rust
c.bench_function("small input", |b| {
    b.iter(|| parse(black_box("short")))
});

c.bench_function("large input", |b| {
    let large = "x".repeat(10_000);
    b.iter(|| parse(black_box(&large)))
});
```

### Parameterized Benchmarks

```rust
use criterion::{BenchmarkId, Criterion};

fn bench_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_sizes");

    for size in [10, 100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |b, &size| {
                let input = "x".repeat(size);
                b.iter(|| parse(black_box(&input)))
            }
        );
    }

    group.finish();
}
```

## Configuration

### Criterion Settings

**`benches/benchmark.rs`:**

```rust
use criterion::{Criterion, SamplingMode};

fn custom_criterion() -> Criterion {
    Criterion::default()
        .sample_size(100)              // More samples = more accurate
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .noise_threshold(0.05)         // 5% noise threshold
        .sampling_mode(SamplingMode::Flat)
}

criterion_group! {
    name = benches;
    config = custom_criterion();
    targets = my_benchmark
}
```

### Workflow Settings

**Adjust duration:**

```yaml
# .github/workflows/benchmark-regression.yml
inputs:
  duration:
    default: '300'  # 5 minutes
```

## Regression Detection

### Threshold Configuration

The workflow detects regressions using:

1. **Statistical significance** (p < 0.05)
2. **Magnitude** (> 5% slower)
3. **Consistency** (median in regression range)

### Manual Threshold

```bash
# Fail if > 10% slower
cargo bench -- --baseline main --significance-level 0.05
```

## Interpreting CI Reports

### PR Comment Example

```markdown
# Benchmark Results

## Performance Summary

| Benchmark | Baseline | Current | Change |
|-----------|----------|---------|--------|
| parse_small | 1.23 µs | 1.20 µs | -2.4% ✅ |
| parse_large | 45.6 µs | 52.3 µs | +14.7% ⚠️ |
| compute | 234 ns | 236 ns | +0.9% ⚪ |

## Regressions Detected ⚠️

**parse_large**: 14.7% slower (p < 0.01)
- Review recent changes to parsing logic
- Consider optimization or accept tradeoff
```

**Action:** Investigate `parse_large` regression.

## Handling Regressions

### Investigate

```bash
# Profile with cargo-flamegraph
cargo install flamegraph
cargo flamegraph --bench performance

# Check with perf
perf record --call-graph dwarf cargo bench
perf report
```

### Options

1. **Fix** - Optimize the code
2. **Accept** - Document tradeoff (e.g., correctness > speed)
3. **Defer** - Create issue for future optimization

### Document Acceptance

```rust
// Intentional tradeoff: Added validation reduces performance by ~10%
// See issue #123 for optimization ideas
fn parse(input: &str) -> Result<Output> {
    validate(input)?;  // New validation (slower but correct)
    // ...
}
```

## Advanced Features

### Compare Multiple Baselines

```bash
# Save different baselines
cargo bench -- --save-baseline main
cargo bench -- --save-baseline before-refactor

# Compare
cargo bench -- --baseline before-refactor
```

### Custom Plots

Criterion generates HTML reports:

```
target/criterion/
├── expensive_function/
│   ├── report/
│   │   ├── index.html
│   │   └── violin.svg
│   └── base/
└── report/
    └── index.html
```

**View:** Open `target/criterion/report/index.html`

### Throughput Measurement

```rust
c.bench_function("process_bytes", |b| {
    let data = vec![0u8; 1_000_000];
    b.throughput(Throughput::Bytes(data.len() as u64));
    b.iter(|| process_data(black_box(&data)))
});
```

**Output:** MB/s instead of time/iteration.

## Troubleshooting

### Noisy Results

```
change: [-15% +2% +18%] (p = 0.52)
Change within noise threshold.
```

**Causes:**
- CPU frequency scaling
- Background processes
- Thermal throttling

**Solutions:**
```bash
# Increase sample size
cargo bench -- --sample-size 1000

# Disable CPU frequency scaling (Linux)
sudo cpupower frequency-set --governor performance
```

### Missing Baseline

```
Warning: No baseline found for benchmark
```

**Fix:** Run once on main branch to establish baseline.

### Slow Benchmarks

```bash
# Reduce measurement time
cargo bench -- --measurement-time 1
```

## Best Practices

1. **Benchmark hot paths** - Focus on critical performance code
2. **Use realistic inputs** - Benchmark with production-like data
3. **Isolate variables** - One change at a time
4. **Accept some variation** - ±5% is often noise
5. **Profile before optimizing** - Use flamegraph/perf
6. **Document tradeoffs** - Sometimes slower is better (safety, correctness)

## Links

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Benchmark Analysis](https://bheisler.github.io/criterion.rs/book/analysis.html)
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

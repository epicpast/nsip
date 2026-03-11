---
diataxis_type: reference
---
# Benchmark Workflow

## Overview

Runs the project's benchmark suite on every push and pull request to `main` to
detect performance regressions early. Results are preserved as a workflow
artifact for 30 days so they can be compared across runs.

**Workflow:** `.github/workflows/benchmark.yml`  
**Trigger:** Push to `main`/`master`, pull requests to `main`/`master`, manual  
**Required secrets:** None

> **Note:** For automated regression detection with baseline comparison, see the
> [Benchmark Regression workflow](BENCHMARK-REGRESSION.md).

## Jobs

### `benchmark`

Runs on `ubuntu-latest` with the stable Rust toolchain.

1. Checks out the repository
2. Configures Rust with caching (cache key: `benchmark`)
3. Executes `cargo bench --workspace` — runs every benchmark in every crate
4. Uploads the Criterion output directory as a workflow artifact

**Artifact:** `benchmark-results` (retained 30 days)  
**Artifact path:** `target/criterion/`

## Running Locally

```bash
# Run all benchmarks
cargo bench --workspace

# Run a specific benchmark by name
cargo bench --workspace -- <bench_name>

# Run and open the HTML report
cargo bench --workspace
open target/criterion/report/index.html
```

## Concurrency

The workflow uses a concurrency group keyed to `${{ github.workflow }}-${{ github.ref }}`.
If a new push to the same ref arrives while a benchmark run is in progress,
the older run is cancelled.

## Interpreting Results

Criterion prints a statistical summary for each benchmark:

```
bench_name         time:   [X.XX ms X.XX ms X.XX ms]
```

The three values are the lower bound, estimate, and upper bound of the
95 % confidence interval. Criterion also reports whether the measured change
is a regression, improvement, or statistically insignificant compared to the
previous baseline stored in `target/criterion/`.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Benchmarks take too long | Large data set or slow algorithm | Profile with `cargo flamegraph` or reduce bench data size |
| `cargo bench` not found | No bench targets defined | Add `[[bench]]` sections to `Cargo.toml` |
| High variance in results | Noisy CI environment | Run locally for accurate results; CI benchmarks are for trend detection only |

See also: [Benchmark Regression workflow](BENCHMARK-REGRESSION.md).

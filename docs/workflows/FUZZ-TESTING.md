---
diataxis_type: reference
---
# Fuzz Testing with cargo-fuzz

## Overview

Automated fuzz testing to discover crashes, panics, and edge cases using [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).

**Workflow:** `.github/workflows/fuzz-testing.yml`
**Tool:** `cargo-fuzz` (libFuzzer)
**Schedule:** Daily at 2 AM
**Goal:** Find unexpected inputs that cause crashes

## How It Works

Fuzz testing generates random/mutated inputs and feeds them to your code:

1. **Generate Inputs**: Create random or mutated test inputs
2. **Execute**: Run target function with inputs
3. **Monitor**: Detect crashes, panics, timeouts, memory errors
4. **Minimize**: Reduce crashing inputs to minimal reproducible cases
5. **Report**: Save crash artifacts for investigation

**Fuzzing finds bugs traditional testing misses.**

## Setup

### Initialize Fuzz Directory

```bash
# Install cargo-fuzz (requires nightly Rust)
cargo install cargo-fuzz

# Initialize fuzz targets
cargo fuzz init

# Directory structure:
# fuzz/
# ├── Cargo.toml
# └── fuzz_targets/
#     └── fuzz_target_1.rs
```

### Create Fuzz Target

**Example: `fuzz/fuzz_targets/parse_input.rs`**

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use nsip::parse;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string
    if let Ok(s) = std::str::from_utf8(data) {
        // Fuzz the parse function
        let _ = parse(s);
    }
});
```

### Structured Fuzzing

For structured data, use `arbitrary`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    value: i32,
    flag: bool,
    data: Vec<u8>,
}

fuzz_target!(|input: FuzzInput| {
    // Fuzz with structured input
    process(input.value, input.flag, &input.data);
});
```

## Usage

### Local Fuzzing

```bash
# List fuzz targets
cargo fuzz list

# Run specific target for 60 seconds
cargo fuzz run parse_input -- -max_total_time=60

# Run with more jobs (parallel)
cargo fuzz run parse_input -- -jobs=4

# Run with corpus (saved inputs)
cargo fuzz run parse_input fuzz/corpus/parse_input
```

### CI Integration

The workflow runs automatically:
- **Daily** at 2 AM (scheduled)
- **Manual** via workflow dispatch
- **Duration:** 5 minutes per target (configurable)

**Crash Detection:**
- Creates GitHub issue if crashes found
- Uploads crash artifacts (90-day retention)

## Understanding Results

### Successful Run (No Crashes)

```
#0  READ units: 1234
#1  pulse  cov: 234 ft: 456 corp: 10/1234b
...
Done 10000 runs in 300 seconds
```

- **units**: Inputs tested
- **cov**: Code coverage
- **ft**: Features covered
- **corp**: Corpus size

### Crash Detected

```
==1234==ERROR: AddressSanitizer: heap-buffer-overflow
READ of size 1 at 0x...
```

**Artifact saved:** `fuzz/artifacts/parse_input/crash-da39a3ee5e6b4b0d3255bfef95601890afd80709`

**Reproduce:**
```bash
cargo fuzz run parse_input fuzz/artifacts/parse_input/crash-*
```

## Crash Investigation

### Reproduce Crash

```bash
# Run with specific crash input
cargo fuzz run target_name crash_artifact
```

### Minimize Crash Input

```bash
# Reduce to minimal crashing input
cargo fuzz tmin target_name crash_artifact
```

### Debug

```rust
// Add to fuzz target for debugging
fuzz_target!(|data: &[u8]| {
    eprintln!("Input length: {}", data.len());
    if let Ok(s) = std::str::from_utf8(data) {
        eprintln!("Input: {:?}", s);
        let _ = parse(s);
    }
});
```

## Common Fuzz Targets

### 1. Parsers

```rust
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parser::parse(s);
    }
});
```

### 2. Deserialization

```rust
fuzz_target!(|data: &[u8]| {
    let _: Result<MyStruct, _> = serde_json::from_slice(data);
});
```

### 3. Binary Protocols

```rust
fuzz_target!(|data: &[u8]| {
    let _ = decode_packet(data);
});
```

### 4. State Machines

```rust
#[derive(Arbitrary, Debug)]
enum Action {
    Start,
    Process(u8),
    Stop,
}

fuzz_target!(|actions: Vec<Action>| {
    let mut state = State::new();
    for action in actions {
        state.handle(action);
    }
});
```

## Corpus Management

### Seed Corpus

Create initial inputs in `fuzz/corpus/target_name/`:

```bash
mkdir -p fuzz/corpus/parse_input
echo "valid input" > fuzz/corpus/parse_input/valid1
echo "" > fuzz/corpus/parse_input/empty
echo "🦀" > fuzz/corpus/parse_input/unicode
```

### Corpus Growth

Fuzzer automatically saves interesting inputs:

```
fuzz/corpus/parse_input/
├── 0a1b2c3d4e5f...  # Auto-generated interesting cases
├── 1b2c3d4e5f6a...
└── seed_inputs/     # Your seed corpus
```

## Configuration

### Adjust Timeout

```yaml
# In workflow
duration: '600'  # 10 minutes
```

### Memory Limits

```bash
# Limit memory usage
cargo fuzz run target -- -rss_limit_mb=2048
```

### Dictionary

Create `fuzz/dict/target.dict` for domain-specific keywords:

```
"keyword1"
"keyword2"
"special_token"
```

```bash
cargo fuzz run target -- -dict=fuzz/dict/target.dict
```

## Troubleshooting

### Slow Fuzzing

```bash
# Run with more jobs
cargo fuzz run target -- -jobs=8

# Reduce input size
cargo fuzz run target -- -max_len=1024
```

### Out of Memory

```bash
# Limit RSS
cargo fuzz run target -- -rss_limit_mb=2048

# Reduce corpus
rm -rf fuzz/corpus/target/*
```

### No New Coverage

Fuzzer might be stuck. Try:

1. **Better seed corpus**: Add diverse initial inputs
2. **Dictionary**: Add domain keywords
3. **Structured fuzzing**: Use `arbitrary` for complex inputs

## Best Practices

1. **Start simple**: Fuzz one function at a time
2. **Use seed corpus**: Guide fuzzer with valid examples
3. **Run long sessions**: Hours or days, not minutes
4. **Minimize crashes**: Use `cargo fuzz tmin` for debugging
5. **Continuous fuzzing**: Run in CI regularly
6. **Multiple targets**: Fuzz different entry points

## Security Benefits

Fuzz testing finds:
- Buffer overflows
- Integer overflows
- Assertion failures
- Panics and unwraps
- Memory leaks
- Logic errors with edge cases

## Example: Complete Fuzz Target

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct Config {
    timeout: u32,
    retries: u8,
    url: String,
}

fuzz_target!(|config: Config| {
    // Validate constraints
    if config.timeout > 0 && config.timeout < 10000 {
        if config.retries <= 10 {
            if config.url.len() < 256 {
                // Fuzz the actual function
                let _ = process_request(&config);
            }
        }
    }
});

fn process_request(config: &Config) -> Result<(), Error> {
    // Implementation
    Ok(())
}
```

## Links

- [cargo-fuzz Book](https://rust-fuzz.github.io/book/)
- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [Arbitrary Crate](https://docs.rs/arbitrary/)
- [Fuzzing Rust Code](https://rust-fuzz.github.io/book/introduction.html)

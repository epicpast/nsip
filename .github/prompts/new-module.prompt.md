---
mode: ask
description: Scaffold a new Rust module with lib.rs export, tests, and documentation
---

# New Module

Create a new Rust module for this project.

## Inputs

- **Module name**: The name for the new module (snake_case)
- **Purpose**: Brief description of what the module does

## Steps

1. Create `crates/{module_name}.rs` with:
   - Module-level doc comment explaining purpose
   - Public types and functions with full documentation
   - `# Examples` and `# Errors` sections on all public items
   - Error type using `thiserror` if the module has fallible operations
2. Add `pub mod {module_name};` to `crates/lib.rs`
3. Re-export key public items from `crates/lib.rs` if appropriate
4. Add a `#[cfg(test)] mod tests` block with:
   - At least one success-path test
   - At least one error-path test
5. Ensure no `unwrap()`, `expect()`, or `panic!()` in the module
6. Run `cargo clippy --all-targets --all-features -- -D warnings`
7. Run `cargo test`

---
mode: ask
description: Fix clippy warnings following project lint configuration
---

# Fix Clippy Warnings

Fix all clippy warnings in the project.

## Steps

1. Run `cargo clippy --all-targets --all-features -- -D warnings 2>&1`
2. For each warning, apply the fix following project conventions:
   - Replace `unwrap()` / `expect()` with proper `Result` handling
   - Replace `todo!()` / `unimplemented!()` with actual implementations
   - Remove `dbg!()` / `print!()` macros
   - Apply suggested clippy fixes for pedantic and nursery lints
3. Do not suppress warnings with `#[allow(...)]` unless there is a
   documented justification
4. Re-run clippy to verify all warnings are resolved
5. Run `cargo test --all-features` to ensure fixes did not break anything

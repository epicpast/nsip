---
mode: ask
description: Add a new thiserror variant with proper attributes
---

# Add Error Variant

Add a new variant to an existing `thiserror` error enum.

## Inputs

- **Error enum**: Which error type to extend (e.g., `Error` in `crates/error.rs`)
- **Variant name**: Name for the new variant (PascalCase)
- **Message**: Human-readable error message for `#[error(...)]`
- **Fields**: Any fields the variant should carry

## Steps

1. Add the new variant to the error enum with `#[error("...")]` attribute
2. If wrapping another error, use `#[from]` or `#[source]` as appropriate:
   - `#[from]` for automatic conversion (one per source type)
   - `#[source]` for manual conversion or multiple variants from same type
3. Update any match arms that handle this error enum exhaustively
4. Add a unit test verifying the error Display output
5. Update doc comments on functions that can now return this variant
6. Run `cargo clippy --all-targets --all-features -- -D warnings`
7. Run `cargo test --all-features`

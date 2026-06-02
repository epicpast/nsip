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
6. Create a catalog page under `docs/reference/errors/<domain>/<slug>.md` for the
   new variant (or `ValidationKind`). CLAUDE.md mandates one catalog page per
   error variant. Mirror an existing page (e.g. `docs/reference/errors/api/not-found.md`)
   and document:
   - the stable `type` URI (`.../docs/reference/errors/<domain>/<slug>.md`)
   - HTTP `status`, `exit_code` (sysexits-aligned), and error `class`
   - the `suggested_fix` applicability marker (e.g. `maybe_incorrect`) and
     whether `retry_after` is set
   - **When it occurs** and **Recovery** guidance, plus an example
     `application/problem+json` envelope
7. Run `cargo clippy --all-targets --all-features -- -D warnings`
8. Run `cargo test --all-features`
9. Run `cargo doc --no-deps` to verify the doc build (including any miette
   `url(...)` links) succeeds

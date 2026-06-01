//! Build script: expose the RFC 9457 error type-URI configuration to the crate.
//!
//! Reads `[package.metadata.nsip]` from `Cargo.toml` so a downstream fork can
//! point the error `type` / `docs_url` URIs at its own documentation without
//! editing source:
//!
//! ```toml
//! [package.metadata.nsip]
//! error-type-uri-base = "https://example.com/errors"
//!
//! [package.metadata.nsip.error-slugs]   # optional, per-error overrides
//! api-timeout = "errors/timeout"
//! ```
//!
//! It emits `NSIP_ERROR_TYPE_URI_BASE` (always) and `NSIP_SLUG_<KEY>` (only for
//! the keys actually present) as compile-time env vars, consumed via
//! `env!` / `option_env!` in `crates/problem.rs` and `crates/lib.rs`.
//!
//! Any read or parse failure falls back to the compiled-in defaults — a missing
//! or malformed manifest section never fails the build.
#![allow(clippy::print_stdout, clippy::print_stderr)]

/// Default type-URI base. MUST stay byte-identical to the value the crate's
/// tests assert; overridden by `[package.metadata.nsip].error-type-uri-base`.
const DEFAULT_TYPE_URI_BASE: &str =
    "https://github.com/zircote/nsip/blob/main/docs/reference/errors";

fn main() {
    println!("cargo::rerun-if-changed=Cargo.toml");
    println!("cargo::rerun-if-changed=build.rs");

    let nsip = read_nsip_metadata();

    let base = nsip
        .as_ref()
        .and_then(|t| t.get("error-type-uri-base"))
        .and_then(toml::Value::as_str)
        .unwrap_or(DEFAULT_TYPE_URI_BASE);
    println!("cargo::rustc-env=NSIP_ERROR_TYPE_URI_BASE={base}");

    // Optional per-error slug overrides. Each key is normalized to the env var
    // `NSIP_SLUG_<UPPER_SNAKE>` consumed by `option_env!` in `problem.rs`; only
    // present keys are emitted, so unspecified slugs keep their in-source default.
    if let Some(slugs) = nsip
        .as_ref()
        .and_then(|t| t.get("error-slugs"))
        .and_then(toml::Value::as_table)
    {
        for (key, value) in slugs {
            if let Some(slug) = value.as_str() {
                let env_key = key.to_uppercase().replace('-', "_");
                println!("cargo::rustc-env=NSIP_SLUG_{env_key}={slug}");
            }
        }
    }
}

/// Parse `[package.metadata.nsip]` from the manifest, returning `None` on any
/// read or parse failure so the build falls back to compiled-in defaults.
fn read_nsip_metadata() -> Option<toml::Value> {
    let dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let path = std::path::Path::new(&dir).join("Cargo.toml");
    let content = std::fs::read_to_string(path).ok()?;
    let manifest: toml::Table = content.parse().ok()?;
    manifest
        .get("package")?
        .get("metadata")?
        .get("nsip")
        .cloned()
}

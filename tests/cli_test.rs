//! CLI integration tests for the `nsip` binary.
//!
//! These tests exercise argument parsing, help output, offline commands
//! (completions, man pages), and error handling. They do NOT make network
//! requests.

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to build a `Command` for the `nsip` binary.
fn nsip_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("nsip")
}

// ---------------------------------------------------------------------------
// Help & version
// ---------------------------------------------------------------------------

#[test]
fn help_flag_exits_zero() {
    nsip_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("NSIP Search API client"));
}

#[test]
fn short_help_flag_exits_zero() {
    nsip_cmd()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn version_flag_exits_zero() {
    nsip_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("nsip"));
}

#[test]
fn short_version_flag_exits_zero() {
    nsip_cmd()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("nsip"));
}

// ---------------------------------------------------------------------------
// Subcommand help
// ---------------------------------------------------------------------------

#[test]
fn search_help() {
    nsip_cmd()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search for animals"));
}

#[test]
fn details_help() {
    nsip_cmd()
        .args(["details", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("detailed information"));
}

#[test]
fn lineage_help() {
    nsip_cmd()
        .args(["lineage", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lineage"));
}

#[test]
fn progeny_help() {
    nsip_cmd()
        .args(["progeny", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("progeny"));
}

#[test]
fn profile_help() {
    nsip_cmd()
        .args(["profile", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("profile"));
}

#[test]
fn compare_help() {
    nsip_cmd()
        .args(["compare", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Compare"));
}

#[test]
fn breed_groups_help() {
    nsip_cmd()
        .args(["breed-groups", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("breed groups"));
}

#[test]
fn statuses_help() {
    nsip_cmd()
        .args(["statuses", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("statuses"));
}

#[test]
fn trait_ranges_help() {
    nsip_cmd()
        .args(["trait-ranges", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("trait ranges"));
}

#[test]
fn date_updated_help() {
    nsip_cmd()
        .args(["date-updated", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("date"));
}

#[test]
fn completions_help() {
    nsip_cmd()
        .args(["completions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn man_pages_help() {
    nsip_cmd()
        .args(["man-pages", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("man page"));
}

#[test]
fn mcp_help() {
    nsip_cmd()
        .args(["mcp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MCP"));
}

// ---------------------------------------------------------------------------
// Completions generation (offline)
// ---------------------------------------------------------------------------

#[test]
fn completions_bash() {
    nsip_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_nsip"));
}

#[test]
fn completions_zsh() {
    nsip_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nsip"));
}

#[test]
fn completions_fish() {
    nsip_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nsip"));
}

#[test]
fn completions_powershell() {
    nsip_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nsip"));
}

#[test]
fn completions_invalid_shell() {
    nsip_cmd()
        .args(["completions", "invalid-shell"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ---------------------------------------------------------------------------
// Man page generation (offline)
// ---------------------------------------------------------------------------

#[test]
fn man_page_stdout() {
    nsip_cmd()
        .arg("man-pages")
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH"));
}

#[test]
fn man_page_to_directory() {
    let dir = tempfile::tempdir().unwrap();
    let dir_path = dir.path().to_str().unwrap();

    nsip_cmd()
        .args(["man-pages", "--out-dir", dir_path])
        .assert()
        .success()
        .stdout(predicate::str::contains("Man pages written to"));

    // Verify main man page exists
    assert!(dir.path().join("nsip.1").exists());
    // Verify at least one subcommand man page exists
    assert!(dir.path().join("nsip-search.1").exists());
}

// ---------------------------------------------------------------------------
// Missing required arguments (error handling)
// ---------------------------------------------------------------------------

#[test]
fn details_missing_arg() {
    nsip_cmd()
        .arg("details")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn lineage_missing_arg() {
    nsip_cmd()
        .arg("lineage")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn progeny_missing_arg() {
    nsip_cmd()
        .arg("progeny")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn profile_missing_arg() {
    nsip_cmd()
        .arg("profile")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn trait_ranges_missing_arg() {
    nsip_cmd()
        .arg("trait-ranges")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn compare_missing_args() {
    nsip_cmd()
        .arg("compare")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn compare_only_one_id() {
    nsip_cmd()
        .args(["compare", "ONLY_ONE"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("2 values required"));
}

#[test]
fn completions_missing_shell() {
    nsip_cmd()
        .arg("completions")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ---------------------------------------------------------------------------
// Invalid subcommand
// ---------------------------------------------------------------------------

#[test]
fn unknown_subcommand() {
    nsip_cmd()
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// No subcommand
// ---------------------------------------------------------------------------

#[test]
fn no_subcommand_emits_error_envelope() {
    // A bare invocation is a usage error. Under the dual-consumer contract it is
    // rendered through the error pipeline (non-TTY → RFC 9457 problem+json on
    // stderr) rather than clap's raw usage text, with a non-zero exit.
    nsip_cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("validation error"))
        .stderr(predicate::str::contains(
            "docs/reference/errors/cli/validation.md",
        ));
}

// ---------------------------------------------------------------------------
// RFC 9457 error envelope (dual-consumer rendering)
// ---------------------------------------------------------------------------

#[test]
fn error_envelope_json_is_well_formed() {
    let assert = nsip_cmd()
        .args(["--format", "json", "bogus-subcommand"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf-8 stderr");
    let pd: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr must be valid problem+json");

    assert_eq!(
        pd["type"],
        "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/validation.md"
    );
    assert_eq!(pd["status"], 400);
    assert_eq!(pd["exit_code"], 1);
    assert!(
        pd["instance"]
            .as_str()
            .is_some_and(|s| s.starts_with("urn:nsip:")),
        "instance must be a urn:nsip: URN, got {:?}",
        pd["instance"]
    );
    assert!(pd["title"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(pd["detail"].as_str().is_some());
    assert!(pd["docs_url"].as_str().is_some());
}

#[test]
fn pretty_format_is_human_not_json() {
    nsip_cmd()
        .args(["--format", "pretty", "bogus-subcommand"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("validation error"))
        .stderr(predicate::str::contains("{\"type\"").not());
}

#[test]
fn json_flag_alias_selects_json_errors() {
    let assert = nsip_cmd().args(["-J", "bogus"]).assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf-8 stderr");
    assert!(
        serde_json::from_str::<serde_json::Value>(stderr.trim()).is_ok(),
        "-J should select JSON error output, got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Global --json / -J flag
// ---------------------------------------------------------------------------

#[test]
fn json_flag_accepted_with_help() {
    nsip_cmd()
        .args(["--json", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn json_short_flag_accepted_with_help() {
    nsip_cmd()
        .args(["-J", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn json_flag_after_subcommand_with_help() {
    // Global flags should work after the subcommand too
    nsip_cmd()
        .args(["search", "-J", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search for animals"));
}

// ---------------------------------------------------------------------------
// Search argument validation
// ---------------------------------------------------------------------------

#[test]
fn search_invalid_page_type() {
    nsip_cmd()
        .args(["search", "--page", "not-a-number"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn search_invalid_page_size_type() {
    nsip_cmd()
        .args(["search", "--page-size", "abc"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn trait_ranges_invalid_breed_id() {
    nsip_cmd()
        .args(["trait-ranges", "not-a-number"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

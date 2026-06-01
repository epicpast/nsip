//! Dual-consumer error rendering for the `nsip` binary.
//!
//! Humans on a TTY get the `miette` graphical diagnostic; agents (or any
//! non-TTY consumer, or an explicit `--format=json`) get the RFC 9457
//! `application/problem+json` envelope on stderr. Mirrors the pattern proven
//! in the sibling `git-creep` CLI.

use std::io::{IsTerminal as _, Write as _};
use std::process::ExitCode;

use clap::ValueEnum;
use clap::error::ErrorKind;

use crate::Commands;

/// Error output format. Auto-detects from the stderr TTY when not specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// `miette` graphical renderer (human-oriented).
    Pretty,
    /// RFC 9457 Problem Details JSON (`application/problem+json`).
    Json,
}

/// Resolve the effective error format. Precedence: explicit `--format` flag,
/// then the legacy `-J/--json` alias, then stderr TTY detection (a non-TTY
/// stderr — pipe, file, agent — defaults to JSON).
#[must_use]
pub fn resolve_format(explicit: Option<Format>, json_flag: bool) -> Format {
    if let Some(f) = explicit {
        return f;
    }
    if json_flag {
        return Format::Json;
    }
    if std::io::stderr().is_terminal() {
        Format::Pretty
    } else {
        Format::Json
    }
}

/// Scan argv for `--format=<v>` / `--format <v>` / `-J` / `--json`. Used when
/// `clap` argument parsing fails (before a `Cli` exists) so the parse-error
/// envelope still honors the caller's requested format; falls back to TTY
/// detection for absent or unrecognized values.
#[must_use]
pub fn detect_format_from_argv() -> Format {
    // `args_os`, not `args`: the latter panics on a non-UTF-8 argv element, and
    // this runs on the error path (a clap parse failure) where a panic would
    // replace the structured error with an abort. A non-UTF-8 arg cannot match
    // our ASCII flags, so it is skipped.
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        let Some(arg) = arg.to_str() else { continue };
        if arg == "-J" || arg == "--json" {
            return Format::Json;
        }
        if let Some(value) = arg.strip_prefix("--format=") {
            return parse_format_value(value);
        }
        if arg == "--format"
            && let Some(value) = args.next().and_then(|v| v.into_string().ok())
        {
            return parse_format_value(&value);
        }
    }
    resolve_format(None, false)
}

fn parse_format_value(raw: &str) -> Format {
    match raw.to_ascii_lowercase().as_str() {
        "json" => Format::Json,
        "pretty" => Format::Pretty,
        // Unknown value: defer to TTY detection rather than silently forcing a
        // mode, keeping non-TTY callers on structured output.
        _ => resolve_format(None, false),
    }
}

/// The subcommand name, used to seed the Problem Details `instance` URN.
#[must_use]
pub const fn command_name(command: &Commands) -> &'static str {
    match command {
        Commands::DateUpdated => "date-updated",
        Commands::BreedGroups => "breed-groups",
        Commands::Statuses => "statuses",
        Commands::TraitRanges { .. } => "trait-ranges",
        Commands::Search { .. } => "search",
        Commands::Details { .. } => "details",
        Commands::Lineage { .. } => "lineage",
        Commands::Progeny { .. } => "progeny",
        Commands::Profile { .. } => "profile",
        Commands::Compare { .. } => "compare",
        Commands::Completions { .. } => "completions",
        Commands::ManPages { .. } => "man-pages",
        Commands::Mcp { .. } => "mcp",
    }
}

/// Extract a concise one-line message from a `clap` parse error for the
/// Problem Details `detail`. Takes the first non-empty line and strips clap's
/// `error: ` prefix.
#[must_use]
pub fn clap_error_message(err: &clap::Error) -> String {
    err.to_string()
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("argument parsing failed")
        .trim_start_matches("error: ")
        .to_owned()
}

/// Whether a `clap` error is an informational display — an explicit `--help`
/// or `--version` — that should print normally and exit success, rather than
/// be rendered as an error envelope. A missing required subcommand is a usage
/// *error* (clap kind `DisplayHelpOnMissingArgumentOrSubcommand`, exit 2) and
/// is intentionally excluded so it flows through the problem+json pipeline.
#[must_use]
pub fn is_clap_display(err: &clap::Error) -> bool {
    matches!(
        err.kind(),
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
    )
}

/// Render `err` in the selected format and return the process exit code.
#[must_use]
pub fn render_and_exit(err: nsip::Error, command: &str, format: Format) -> ExitCode {
    let exit = u8::try_from(err.exit_code()).unwrap_or(1);
    match format {
        Format::Pretty => render_pretty(err),
        Format::Json => render_json(&err, command),
    }
    ExitCode::from(exit)
}

#[allow(
    clippy::print_stderr,
    reason = "renderer is the sanctioned stderr writer"
)]
fn render_pretty(err: nsip::Error) {
    let report = miette::Report::new(err);
    eprintln!("{report:?}");
}

#[allow(
    clippy::print_stderr,
    reason = "renderer is the sanctioned stderr writer"
)]
fn render_json(err: &nsip::Error, command: &str) {
    let pd = err.to_problem_details(command);
    if let Ok(s) = serde_json::to_string(&pd) {
        let mut stderr = std::io::stderr().lock();
        if writeln!(stderr, "{s}").is_err() {
            eprintln!("{s}");
        }
    } else {
        // Hand-built fallback so a serialization failure is never silent.
        let fallback = format!(
            "{{\"type\":\"{type_uri}\",\"title\":\"{title}\",\"status\":{status},\"exit_code\":{exit_code},\"detail\":\"serialization failed\"}}",
            type_uri = err.type_uri(),
            title = err.title(),
            status = err.status_code(),
            exit_code = err.exit_code(),
        );
        eprintln!("{fallback}");
    }
}

#[cfg(test)]
mod tests {
    use std::process::ExitCode;

    use super::*;

    #[test]
    fn resolve_format_precedence() {
        // Explicit flag wins over everything.
        assert_eq!(resolve_format(Some(Format::Pretty), true), Format::Pretty);
        assert_eq!(resolve_format(Some(Format::Json), false), Format::Json);
        // `-J/--json` wins over TTY detection.
        assert_eq!(resolve_format(None, true), Format::Json);
        // No flag in the (non-TTY) test environment defaults to JSON.
        assert_eq!(resolve_format(None, false), Format::Json);
    }

    #[test]
    fn parse_format_value_cases() {
        assert_eq!(parse_format_value("json"), Format::Json);
        assert_eq!(parse_format_value("JSON"), Format::Json);
        assert_eq!(parse_format_value("pretty"), Format::Pretty);
        assert_eq!(parse_format_value("Pretty"), Format::Pretty);
        // Unknown value falls back to TTY detection (JSON in the test env).
        assert_eq!(parse_format_value("nonsense"), Format::Json);
    }

    #[test]
    fn detect_format_from_argv_no_flag() {
        // The test harness argv carries no `--format`; falls back to detection.
        assert_eq!(detect_format_from_argv(), Format::Json);
    }

    #[test]
    fn command_name_maps_variants() {
        assert_eq!(command_name(&Commands::DateUpdated), "date-updated");
        assert_eq!(command_name(&Commands::BreedGroups), "breed-groups");
        assert_eq!(command_name(&Commands::Statuses), "statuses");
        assert_eq!(
            command_name(&Commands::TraitRanges { breed_id: 1 }),
            "trait-ranges"
        );
        assert_eq!(
            command_name(&Commands::Details {
                search_string: "x".into(),
            }),
            "details"
        );
        assert_eq!(
            command_name(&Commands::Lineage { lpn_id: "x".into() }),
            "lineage"
        );
        assert_eq!(
            command_name(&Commands::Progeny {
                lpn_id: "x".into(),
                page: 1,
                page_size: 10,
            }),
            "progeny"
        );
        assert_eq!(
            command_name(&Commands::Profile { lpn_id: "x".into() }),
            "profile"
        );
        assert_eq!(
            command_name(&Commands::Compare {
                lpn_ids: vec!["a".into()],
                traits: None,
            }),
            "compare"
        );
        assert_eq!(
            command_name(&Commands::Completions {
                shell: clap_complete::Shell::Bash,
            }),
            "completions"
        );
        assert_eq!(
            command_name(&Commands::ManPages { out_dir: None }),
            "man-pages"
        );
        assert_eq!(
            command_name(&Commands::Mcp {
                transport: "stdio".into(),
                host: "127.0.0.1".into(),
                port: 0,
                tools: None,
                auth: false,
            }),
            "mcp"
        );
        assert_eq!(
            command_name(&Commands::Search {
                breed_id: None,
                breed_group_id: None,
                status: None,
                gender: None,
                born_after: None,
                born_before: None,
                proven_only: false,
                flock_id: None,
                sort_by: None,
                reverse: false,
                page: 1,
                page_size: 10,
            }),
            "search"
        );
    }

    #[test]
    fn clap_error_message_strips_prefix() {
        let err = clap::Error::raw(ErrorKind::InvalidValue, "error: bad value\nmore detail\n");
        assert_eq!(clap_error_message(&err), "bad value");
    }

    #[test]
    fn is_clap_display_only_help_and_version() {
        assert!(is_clap_display(&clap::Error::raw(
            ErrorKind::DisplayHelp,
            "h"
        )));
        assert!(is_clap_display(&clap::Error::raw(
            ErrorKind::DisplayVersion,
            "v"
        )));
        assert!(!is_clap_display(&clap::Error::raw(
            ErrorKind::InvalidValue,
            "x"
        )));
        // A missing subcommand is a usage error, NOT a clean display.
        assert!(!is_clap_display(&clap::Error::raw(
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand,
            "x"
        )));
    }

    #[test]
    fn render_and_exit_returns_variant_exit_code() {
        // Both render paths execute (writing to the test's stderr) and the exit
        // code reflects the variant's classification.
        let json_code = render_and_exit(nsip::Error::api(503, "down"), "test", Format::Json);
        let pretty_code = render_and_exit(nsip::Error::validation("bad"), "test", Format::Pretty);
        // EX_TEMPFAIL (75) for transient, 1 for caller error.
        assert_eq!(
            format!("{json_code:?}"),
            format!("{:?}", ExitCode::from(75))
        );
        assert_eq!(
            format!("{pretty_code:?}"),
            format!("{:?}", ExitCode::from(1))
        );
    }
}

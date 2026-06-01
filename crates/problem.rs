//! RFC 9457 Problem Details envelope (`application/problem+json`).
//!
//! Every `Error` maps to a [`ProblemDetails`] object so that LLM
//! agents orchestrating `nsip` (via the CLI or the MCP server) receive a
//! spec-exact, machine-parseable error contract instead of a bare string.
//! Humans keep the `miette` graphical rendering (see `crates/render.rs`).
//!
//! The struct is hand-rolled rather than depending on the `rfc9457` crate
//! (0.1.0 as of 2026-06): the five RFC 9457 core members plus the
//! `nsip`-specific extensions are the complete public surface. Applicability
//! markers live in the documentation catalog (`docs/reference/ERRORS.md`)
//! keyed by `type`, not inline, to hold the envelope under the 1 KB cap.
//!
//! See [`Error::to_problem_details`](crate::Error::to_problem_details) for the
//! variant → envelope mapping and `docs/reference/ERROR-ENVELOPE.md` for the
//! field schema.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Error, ValidationKind};

/// `<domain>/<slug>` path for a [`ValidationKind`]; the per-operation problem type.
const fn validation_slug(kind: ValidationKind) -> &'static str {
    match kind {
        ValidationKind::EmptyLpnId => "cli/empty-lpn-id",
        ValidationKind::InvalidBreedId => "cli/invalid-breed-id",
        ValidationKind::PageRange => "cli/page-range",
        ValidationKind::EmptySearch => "cli/empty-search",
        ValidationKind::CompareArity => "cli/compare-arity",
        ValidationKind::MissingArgument => "mcp/missing-argument",
        ValidationKind::UnknownResource => "mcp/unknown-resource",
        ValidationKind::InvalidCursor => "mcp/invalid-cursor",
        ValidationKind::UnknownTransport => "cli/unknown-transport",
        ValidationKind::Other => "cli/validation",
    }
}

/// Stable, type-level title for a [`ValidationKind`].
const fn validation_title(kind: ValidationKind) -> &'static str {
    match kind {
        ValidationKind::EmptyLpnId => "LPN ID must not be empty",
        ValidationKind::InvalidBreedId => "Breed ID is invalid",
        ValidationKind::PageRange => "Pagination parameter out of range",
        ValidationKind::EmptySearch => "Search request has no filter",
        ValidationKind::CompareArity => "Comparison requires 2 to 5 animals",
        ValidationKind::MissingArgument => "Required argument is missing",
        ValidationKind::UnknownResource => "Unknown resource URI",
        ValidationKind::InvalidCursor => "Invalid pagination cursor",
        ValidationKind::UnknownTransport => "Unknown MCP transport",
        ValidationKind::Other => "Invalid input parameters",
    }
}

/// Tailored, per-operation `suggested_fix` for a [`ValidationKind`]. Every kind
/// has a deterministic fix; applicability markers are catalogued in
/// `docs/reference/ERRORS.md` keyed by `type`.
fn validation_fix(kind: ValidationKind, message: &str) -> String {
    match kind {
        ValidationKind::EmptyLpnId => "provide a non-empty LPN ID".to_owned(),
        ValidationKind::InvalidBreedId => {
            "provide a positive integer breed id (see the breed_groups tool)".to_owned()
        },
        ValidationKind::PageRange => "use page >= 1 and page_size between 1 and 100".to_owned(),
        ValidationKind::EmptySearch => {
            "provide a non-empty query (an LPN ID or name) or at least one search filter".to_owned()
        },
        ValidationKind::CompareArity => "pass between 2 and 5 LPN IDs".to_owned(),
        ValidationKind::MissingArgument => format!("provide the required argument: {message}"),
        ValidationKind::UnknownResource => {
            "use a documented nsip:// resource URI (see nsip://glossary)".to_owned()
        },
        ValidationKind::InvalidCursor => {
            "restart pagination without a cursor (begin from the first page)".to_owned()
        },
        ValidationKind::UnknownTransport => "use --transport stdio or --transport http".to_owned(),
        ValidationKind::Other => format!("correct the input and retry: {message}"),
    }
}

/// Stable base for `type`/`docs_url` URIs. Per the committed policy the URI is
/// stable forever (no path version); semantic changes are tracked in the
/// documentation changelog. See `docs/adr` for the rationale.
const TYPE_URI_BASE: &str = "https://github.com/zircote/nsip/blob/main/docs/reference/errors";

/// Maximum byte length of the envelope `detail`. The `Api` variant's message is
/// the raw upstream response body (unbounded — see `client.rs`), so it is
/// truncated here to keep the whole envelope within the ~1 KB budget the module
/// documents. The other members (type, title, instance, fixed-string fixes) are
/// short and bounded by construction.
const MAX_DETAIL_LEN: usize = 480;

/// Truncate `detail` to at most [`MAX_DETAIL_LEN`] bytes on a `char` boundary,
/// appending an ellipsis when truncation occurs. Never splits a multi-byte
/// `char` (which would yield invalid UTF-8).
fn truncate_detail(detail: &str) -> String {
    if detail.len() <= MAX_DETAIL_LEN {
        return detail.to_owned();
    }
    let mut end = MAX_DETAIL_LEN;
    while !detail.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &detail[..end])
}

/// RFC 9457 Problem Details object (`application/problem+json`).
///
/// The five RFC 9457 standard members (`type`, `title`, `status`, `detail`,
/// `instance`) plus the agent extensions (`exit_code`, `suggested_fix`,
/// `code_actions`, `retry_after`, `docs_url`). Empty/absent optional members
/// are omitted from the JSON for token economy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    /// Stable URI identifying the problem type. The agent's dispatch key.
    #[serde(rename = "type")]
    pub type_uri: String,

    /// One-sentence, human-readable summary of the problem type (stable per `type`).
    pub title: String,

    /// HTTP-class status: 4xx for caller/upstream-client errors, 5xx for
    /// transient/environment failures.
    pub status: u16,

    /// One to three sentences specific to this occurrence.
    pub detail: String,

    /// Per-occurrence correlation handle, `urn:nsip:<command>:<uuid>`.
    pub instance: String,

    /// Process exit code (`sysexits.h`-aligned where applicable).
    pub exit_code: i32,

    /// Free-text recovery action. Omitted when no deterministic fix exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_fix: Option<String>,

    /// LSP-style structured edits. Omitted when empty (the common case).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub code_actions: Vec<CodeAction>,

    /// Delta-seconds or RFC 3339 timestamp. Populated only on transient errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<RetryAfter>,

    /// Stable documentation URL for this problem type (same as `type` by default).
    pub docs_url: String,
}

/// LSP-style code action carrying a structured edit suggestion.
///
/// Reserved for future use (e.g. corrected CLI flags); the current error set
/// emits no structured edits, so the field is omitted from serialized output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAction {
    /// Human-readable title for the action.
    pub title: String,
    /// LSP `CodeActionKind` (e.g. `"quickfix"`).
    pub kind: Cow<'static, str>,
    /// The structured edit payload.
    pub edit: serde_json::Value,
    /// Whether the agent should prefer this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_preferred: Option<bool>,
}

/// Either a delta-seconds duration or an absolute RFC 3339 timestamp, matching
/// the two forms of the HTTP `Retry-After` header (RFC 7231 §7.1.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RetryAfter {
    /// Delta-seconds: retry is safe after this many seconds.
    Seconds(u32),
    /// Absolute RFC 3339 timestamp after which retry is safe.
    Timestamp(String),
}

impl Error {
    /// `<domain>/<slug>` path component for this error's type URI, e.g.
    /// `api/timeout`. One slug per variant (the `Api` variant carries its HTTP
    /// status in the `status` field rather than splitting into sub-slugs).
    #[must_use]
    pub const fn slug_path(&self) -> &'static str {
        match self {
            Self::Validation { kind, .. } => validation_slug(*kind),
            Self::Api { .. } => "api/error",
            Self::NotFound(_) => "api/not-found",
            Self::Timeout { .. } => "api/timeout",
            Self::Connection { .. } => "api/connection",
            Self::Parse { .. } => "api/upstream-parse",
        }
    }

    /// Stable type URI under the repository's `docs/reference/errors/` tree.
    #[must_use]
    pub fn type_uri(&self) -> String {
        format!("{TYPE_URI_BASE}/{}.md", self.slug_path())
    }

    /// One-sentence, type-level title (stable across occurrences).
    #[must_use]
    pub const fn title(&self) -> &'static str {
        match self {
            Self::Validation { kind, .. } => validation_title(*kind),
            Self::Api { .. } => "Upstream API returned an error",
            Self::NotFound(_) => "Requested resource was not found",
            Self::Timeout { .. } => "Request to the NSIP API timed out",
            Self::Connection { .. } => "Could not connect to the NSIP API",
            Self::Parse { .. } => "Could not parse the NSIP API response",
        }
    }

    /// Process exit code for this error.
    ///
    /// Aligned with `sysexits.h` where applicable:
    /// * `1` — caller error (bad input, 4xx, not found).
    /// * `3` — environment error (unparseable upstream payload).
    /// * `75` — `EX_TEMPFAIL`, transient (timeout, connection, 429, 5xx);
    ///   `retry_after` is populated where a delay is known.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        self.exit_and_status().0
    }

    /// HTTP-class status for the Problem Details envelope.
    #[must_use]
    pub const fn status_code(&self) -> u16 {
        self.exit_and_status().1
    }

    /// Joint `(exit_code, status)` lookup. The `Api` variant reflects the real
    /// upstream HTTP status and classifies 429/5xx as transient.
    const fn exit_and_status(&self) -> (i32, u16) {
        match self {
            Self::Validation { .. } => (1, 400),
            Self::NotFound(_) => (1, 404),
            Self::Parse { .. } => (3, 502),
            Self::Timeout { .. } => (75, 504),
            Self::Connection { .. } => (75, 503),
            Self::Api { status, .. } => {
                if *status == 429 || *status >= 500 {
                    (75, *status)
                } else {
                    (1, *status)
                }
            },
        }
    }

    /// Free-text recovery action for this error, or `None` when there is no
    /// deterministic fix. Applicability markers for each fix are catalogued in
    /// `docs/reference/ERRORS.md`, keyed by `type`.
    #[must_use]
    pub fn suggested_fix(&self) -> Option<String> {
        let s = match self {
            Self::Validation { kind, message } => return Some(validation_fix(*kind, message)),
            Self::NotFound(_) => {
                "verify the identifier exists in the NSIP database (try `nsip search`)".to_owned()
            },
            Self::Timeout { .. } => {
                "retry the request; increase the client timeout if this persists".to_owned()
            },
            Self::Connection { .. } => {
                "check network connectivity to nsipsearch.nsip.org and retry".to_owned()
            },
            Self::Api { status, .. } if *status == 429 => {
                "wait for the retry_after interval before retrying".to_owned()
            },
            Self::Api { status, .. } if *status >= 500 => {
                "the NSIP API is failing; retry after a short delay".to_owned()
            },
            // 4xx (non-429) client errors and upstream parse failures have no
            // deterministic local fix.
            _ => return None,
        };
        Some(s)
    }

    /// Retry delay for transient errors, sourced from the upstream
    /// `Retry-After` header where available. `None` for terminal errors.
    #[must_use]
    pub fn retry_after(&self) -> Option<RetryAfter> {
        match self {
            Self::Api { retry_after, .. }
            | Self::Timeout { retry_after, .. }
            | Self::Connection { retry_after, .. } => retry_after.clone(),
            Self::Validation { .. } | Self::NotFound(_) | Self::Parse { .. } => None,
        }
    }

    /// Build the RFC 9457 Problem Details object for this error.
    ///
    /// `command` seeds the `instance` URN — pass the CLI subcommand or MCP tool
    /// name (e.g. `"search"`), or `"nsip"` if unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = nsip::Error::NotFound("LPN 123".to_string());
    /// let pd = err.to_problem_details("details");
    /// assert_eq!(pd.status, 404);
    /// assert!(pd.instance.starts_with("urn:nsip:details:"));
    /// ```
    #[must_use]
    pub fn to_problem_details(&self, command: &str) -> ProblemDetails {
        let type_uri = self.type_uri();
        let instance = format!("urn:nsip:{command}:{}", Uuid::new_v4());
        ProblemDetails {
            docs_url: type_uri.clone(),
            type_uri,
            title: self.title().to_owned(),
            status: self.status_code(),
            detail: truncate_detail(&self.to_string()),
            instance,
            exit_code: self.exit_code(),
            suggested_fix: self.suggested_fix(),
            code_actions: Vec::new(),
            retry_after: self.retry_after(),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests may panic on setup failure"
)]
mod tests {
    use super::*;

    /// One representative of every variant, for exhaustive envelope checks.
    fn all_variants() -> Vec<Error> {
        vec![
            Error::validation("bad breed_id"),
            Error::api(404, "missing"),
            Error::api(429, "slow down"),
            Error::api(503, "upstream down"),
            Error::not_found("LPN 999"),
            Error::timeout("30s exceeded"),
            Error::connection("refused"),
            Error::parse("bad json"),
        ]
    }

    /// Every variant produces a complete, spec-valid envelope with a unique
    /// `urn:nsip:` instance.
    #[test]
    fn envelope_populated_for_every_variant() {
        let mut seen = std::collections::HashSet::new();
        for err in all_variants() {
            let pd = err.to_problem_details("test");
            assert!(
                pd.type_uri.starts_with(
                    "https://github.com/zircote/nsip/blob/main/docs/reference/errors/"
                ),
                "type_uri: {}",
                pd.type_uri
            );
            // Slug ends in `<word>.md`; check the suffix via the last 3 chars
            // rather than `ends_with(".md")` to avoid clippy's path-extension lint.
            assert_eq!(&pd.type_uri[pd.type_uri.len() - 3..], ".md");
            assert!(!pd.title.is_empty());
            assert!(pd.status >= 400);
            assert!(!pd.detail.is_empty());
            assert!(pd.instance.starts_with("urn:nsip:test:"));
            assert!(pd.exit_code > 0);
            assert_eq!(pd.docs_url, pd.type_uri);
            assert!(
                seen.insert(pd.instance.clone()),
                "instance URN must be unique per call"
            );
        }
    }

    /// Exit-code / status map matches the committed catalog table.
    #[test]
    fn exit_and_status_map_matches_catalog() {
        assert_eq!(Error::validation("x").exit_code(), 1);
        assert_eq!(Error::validation("x").status_code(), 400);
        assert_eq!(Error::not_found("x").status_code(), 404);
        assert_eq!(Error::parse("x").exit_code(), 3);
        assert_eq!(Error::parse("x").status_code(), 502);
        assert_eq!(Error::timeout("x").exit_code(), 75);
        assert_eq!(Error::timeout("x").status_code(), 504);
        assert_eq!(Error::connection("x").exit_code(), 75);
        assert_eq!(Error::connection("x").status_code(), 503);
        // Api reflects the real upstream status; 4xx terminal, 429/5xx transient.
        assert_eq!(Error::api(400, "x").exit_code(), 1);
        assert_eq!(Error::api(429, "x").exit_code(), 75);
        assert_eq!(Error::api(503, "x").exit_code(), 75);
        assert_eq!(Error::api(418, "x").status_code(), 418);
    }

    /// Transient variants surface a populated `retry_after`; terminal ones do not.
    #[test]
    fn retry_after_only_on_transient() {
        let transient = Error::Api {
            status: 429,
            message: "rate limited".to_owned(),
            retry_after: Some(RetryAfter::Seconds(30)),
            source: None,
        };
        let pd = transient.to_problem_details("search");
        assert_eq!(pd.retry_after, Some(RetryAfter::Seconds(30)));

        // Terminal classes never carry a retry hint.
        assert!(
            Error::validation("x")
                .to_problem_details("x")
                .retry_after
                .is_none()
        );
        assert!(
            Error::not_found("x")
                .to_problem_details("x")
                .retry_after
                .is_none()
        );
        assert!(
            Error::parse("x")
                .to_problem_details("x")
                .retry_after
                .is_none()
        );
    }

    /// Empty `code_actions` is omitted from the JSON for token economy, and the
    /// envelope stays under the 1 KB cap.
    #[test]
    fn json_is_compact() {
        for err in all_variants() {
            let pd = err.to_problem_details("cmd");
            let json = serde_json::to_string(&pd).expect("serialize");
            assert!(
                !json.contains("\"code_actions\""),
                "empty code_actions should be omitted: {json}"
            );
            assert!(
                json.len() <= 1024,
                "payload {} bytes exceeds 1 KB cap",
                json.len()
            );
        }
    }

    /// Wrapped variants preserve the originating cause via `source()`.
    #[test]
    fn cause_chain_preserved() {
        let io = std::io::Error::new(std::io::ErrorKind::TimedOut, "underlying");
        let err = Error::Parse {
            message: "failed to parse response".to_owned(),
            source: Some(Box::new(io)),
        };
        let cause = std::error::Error::source(&err);
        assert!(cause.is_some(), "Parse with source must expose source()");
        assert!(cause.unwrap().to_string().contains("underlying"));

        // A constructor-built error (no upstream) has no source — that's fine.
        assert!(std::error::Error::source(&Error::validation("x")).is_none());
    }

    /// Every `ValidationKind` yields a distinct, well-formed envelope: a
    /// `cli/` or `mcp/` slug, a non-empty tailored title and fix, status 400,
    /// exit code 1, and no retry hint.
    #[test]
    fn every_validation_kind_maps_cleanly() {
        let kinds = [
            ValidationKind::EmptyLpnId,
            ValidationKind::InvalidBreedId,
            ValidationKind::PageRange,
            ValidationKind::EmptySearch,
            ValidationKind::CompareArity,
            ValidationKind::MissingArgument,
            ValidationKind::UnknownResource,
            ValidationKind::InvalidCursor,
            ValidationKind::UnknownTransport,
            ValidationKind::Other,
        ];
        let mut slugs = std::collections::HashSet::new();
        for kind in kinds {
            let err = Error::validation_kind(kind, "field");
            let pd = err.to_problem_details("op");
            let slug = err.slug_path();
            assert!(
                slug.starts_with("cli/") || slug.starts_with("mcp/"),
                "{kind:?} slug: {slug}"
            );
            assert!(slugs.insert(slug), "duplicate slug for {kind:?}: {slug}");
            assert_eq!(pd.status, 400);
            assert_eq!(pd.exit_code, 1);
            assert!(!pd.title.is_empty());
            assert!(pd.suggested_fix.is_some(), "{kind:?} must have a fix");
            assert!(pd.retry_after.is_none());
        }
    }

    /// `retry_after` round-trips through both JSON forms (untagged enum).
    #[test]
    fn retry_after_serde_forms() {
        let secs = serde_json::to_string(&RetryAfter::Seconds(12)).unwrap();
        assert_eq!(secs, "12");
        let ts =
            serde_json::to_string(&RetryAfter::Timestamp("2026-06-01T00:00:00Z".into())).unwrap();
        assert_eq!(ts, "\"2026-06-01T00:00:00Z\"");
    }

    /// A verbose upstream body (the `Api` message is the raw response body, which
    /// is unbounded) is truncated so the envelope stays within the 1 KB budget.
    /// `json_is_compact` only ever feeds tiny fixtures, so it does not exercise
    /// truncation — this does.
    #[test]
    fn detail_truncated_keeps_envelope_under_cap() {
        let err = Error::Api {
            status: 500,
            message: "x".repeat(5000),
            retry_after: None,
            source: None,
        };
        let pd = err.to_problem_details("date-updated");
        assert!(
            pd.detail.len() <= MAX_DETAIL_LEN + 4,
            "detail not truncated: {} bytes",
            pd.detail.len()
        );
        assert!(
            pd.detail.ends_with('…'),
            "truncated detail should end with an ellipsis: {}",
            pd.detail
        );
        let json = serde_json::to_string(&pd).expect("serialize");
        assert!(
            json.len() <= 1024,
            "envelope {} bytes exceeds 1 KB cap",
            json.len()
        );
    }

    /// The `type` URI is declared twice — `lib.rs` `#[diagnostic(url(...))]` (the
    /// human/miette path) and `problem.rs` `type_uri()` (the agent envelope). For
    /// every non-`Validation` variant they must resolve to the SAME doc page;
    /// pin them together so the two sources cannot drift silently.
    #[test]
    fn miette_url_matches_envelope_type_uri() {
        use miette::Diagnostic as _;
        for err in [
            Error::api(500, "x"),
            Error::not_found("x"),
            Error::timeout("x"),
            Error::connection("x"),
            Error::parse("x"),
        ] {
            let url = err.url().map(|u| u.to_string());
            assert_eq!(
                url.as_deref(),
                Some(err.type_uri().as_str()),
                "miette url and envelope type diverged for {err:?}"
            );
        }

        // `Validation` is intentionally NOT pinned to equality: a variant-level
        // miette `url` attribute cannot see `kind`, so the human path carries one
        // coarse `cli/validation.md` while the envelope carries the precise
        // per-operation slug. Pin that intended divergence so it stays a
        // documented decision rather than an accident.
        let empty = Error::empty_lpn_id();
        assert_eq!(
            empty.url().map(|u| u.to_string()).as_deref(),
            Some(
                "https://github.com/zircote/nsip/blob/main/docs/reference/errors/cli/validation.md"
            )
        );
        assert!(empty.type_uri().ends_with("/cli/empty-lpn-id.md"));
    }
}

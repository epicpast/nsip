---
diataxis_type: reference
---
# Configuration Reference

Complete reference for configuring the `nsip` CLI and library.

---

## Client Configuration

The `NsipClient` is configured through its builder or constructor methods. There are no environment variables or configuration files.

### Defaults

| Setting | Default value | Description |
|---------|---------------|-------------|
| Base URL | `http://nsipsearch.nsip.org/api` | NSIP Search API endpoint (HTTP-only; the upstream API has no valid TLS certificate) |
| Timeout | 30 seconds | Per-request timeout |
| Max retries | 3 | Automatic retries on server errors (HTTP 500, 502, 503, 504) |
| Backoff factor | 0.5 | Retry delay multiplier (not configurable) |
| Retry delay | `0.5 * 2^attempt` seconds | Exponential backoff formula |

### Constructor Methods

Three ways to create a client:

```rust
use nsip::NsipClient;

// 1. All defaults
let client = NsipClient::new();

// 2. Custom base URL, default timeout and retries
let client = NsipClient::with_base_url("http://localhost:8080/api");

// 3. Full control via builder
let client = NsipClient::builder()
    .base_url("http://localhost:8080/api")
    .timeout_secs(60)
    .max_retries(5)
    .build()?;
```

### Builder Options

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `base_url(url)` | `impl Into<String>` | `http://nsipsearch.nsip.org/api` | API base URL |
| `timeout_secs(secs)` | `u64` | 30 | Request timeout in seconds |
| `max_retries(retries)` | `u32` | 3 | Max retries for 5xx errors |

The `build()` method returns `Result<NsipClient>`. It returns `Error::Connection` if the underlying `reqwest::Client` cannot be constructed (rare in practice).

---

## Retry Policy

The client automatically retries failed requests that receive specific HTTP status codes.

**Retried status codes:** 500, 502, 503, 504

**Backoff schedule:** Exponential with a factor of 0.5.

| Attempt | Delay |
|---------|-------|
| 1 | 0.5 seconds |
| 2 | 1.0 seconds |
| 3 | 2.0 seconds |
| 4 | 4.0 seconds |
| 5 | 8.0 seconds |

After exhausting all retries, the final server error is returned as `Error::Api`.

**Disable retries:**

```rust
let client = NsipClient::builder()
    .max_retries(0)
    .build()?;
```

---

## CLI Configuration

The CLI binary has no configuration file. All options are provided as command-line flags and arguments.

### Global Flags

| Flag | Short | Value | Description |
|------|-------|-------|-------------|
| `--json` | `-J` | -- | Output raw JSON instead of human-readable ASCII tables (alias for `--format json`) |
| `--format` | -- | `pretty` \| `json` | Output format for both success and error output. Defaults to TTY detection. |

These flags are global and apply to all subcommands. `--format` controls both
success output and error rendering: `json` emits JSON plus the RFC 9457
`application/problem+json` envelope on error; `pretty` emits human-readable
output plus a `miette` diagnostic. When omitted, the format is detected from the
stderr TTY (interactive terminal → `pretty`, non-TTY → `json`). An explicit
`--format` takes precedence over `-J/--json`.

### Pagination Defaults

| Command | Default page | Default page_size |
|---------|-------------|-------------------|
| `search` | 0 | 15 |
| `progeny` | 0 | 10 |

---

## MCP Server Configuration

The MCP server supports two transports (stdio and HTTP), optional tool set filtering, OAuth authentication, and telemetry. See [MCP Server Configuration Reference](MCP-SERVER-CONFIGURATION.md) for the complete reference.

### Quick Start

```bash
# Stdio (default) — all tools, no auth
nsip mcp

# HTTP with tool filtering
nsip mcp --transport http --port 8080 --tools search,breed

# HTTP with OAuth authentication
nsip mcp --transport http --port 8080 --auth
```

### OAuth Environment Variables

Required when `--auth` is set:

| Variable | Description |
|----------|-------------|
| `NSIP_GITHUB_CLIENT_ID` | GitHub OAuth app client ID |
| `NSIP_GITHUB_CLIENT_SECRET` | GitHub OAuth app client secret |
| `NSIP_AUTH_SECRET` | HMAC-SHA256 secret for JWT signing |
| `NSIP_AUTH_BASE_URL` | External base URL (e.g. `http://localhost:8080`) |

Optional:

| Variable | Default | Description |
|----------|---------|-------------|
| `NSIP_AUTH_ISSUER` | `https://localhost` | JWT `iss` claim value |
| `NSIP_AUTH_TOKEN_TTL` | `3600` | JWT token time-to-live in seconds |
| `NSIP_AUTH_ALLOWED_USERS` | *(none)* | Comma-separated allowlist of GitHub usernames |

### Telemetry

When compiled with `--features telemetry`, the server logs in JSON format with W3C trace context (`trace_id`, `span_id`). Default build uses plain text tracing.

### Client Configuration

#### Claude Code (`.mcp.json`)

Place at your project root or `~/.mcp.json`:

```json
{
  "mcpServers": {
    "nsip": {
      "command": "nsip",
      "args": ["mcp"]
    }
  }
}
```

#### Claude Desktop (`claude_desktop_config.json`)

```json
{
  "mcpServers": {
    "nsip": {
      "command": "nsip",
      "args": ["mcp"]
    }
  }
}
```

On macOS the config file is at `~/Library/Application Support/Claude/claude_desktop_config.json`.

#### Docker Transport

```json
{
  "mcpServers": {
    "nsip": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "ghcr.io/zircote/nsip", "mcp"]
    }
  }
}
```

---

## Search Criteria Defaults

When using `SearchCriteria` programmatically, all fields default to `None` (no filter applied). The `search` CLI subcommand applies these defaults:

| Option | Default |
|--------|---------|
| `--page` | 0 |
| `--page-size` | 15 |
| `--proven-only` | false |
| `--reverse` | false |
| All other filters | not set (no filtering) |

---

## Validation Constraints

These constraints are enforced at the client level before any API request is made:

| Parameter | Constraint | Error |
|-----------|-----------|-------|
| `breed_id` (for `trait_ranges`) | Must be > 0 | `Error::Validation` |
| `page_size` (for `search_animals`) | Must be 1-100 | `Error::Validation` |
| `page_size` (for `progeny`) | Must be > 0 | `Error::Validation` |
| `search_string` (for `animal_details`) | Must not be empty/whitespace | `Error::Validation` |
| `lpn_id` (for `lineage`) | Must not be empty/whitespace | `Error::Validation` |
| `lpn_id` (for `progeny`) | Must not be empty | `Error::Validation` |
| `lpn_id` (for `search_by_lpn`) | Must not be empty/whitespace | `Error::Validation` |
| `lpn_ids` (for CLI `compare`) | Must provide 2-5 IDs | Clap argument validation |

---

## API Endpoint Mapping

The client translates method calls to these NSIP API endpoints:

| Client method | HTTP method | API path |
|--------------|-------------|----------|
| `date_last_updated()` | GET | `search/getDateLastUpdated` |
| `breed_groups()` | GET | `search/getAvailableBreedGroups` |
| `statuses()` | GET | `search/getStatusesByBreedGroup` |
| `trait_ranges(breed_id)` | GET | `search/getTraitRangesByBreed?breedId={id}` |
| `search_animals(...)` | POST | `search/getPageOfSearchResults` |
| `animal_details(search_string)` | GET | `details/getAnimalDetails?searchString={s}` |
| `lineage(lpn_id)` | GET | `details/getLineage?lpnId={id}` |
| `progeny(lpn_id, ...)` | GET | `details/getPageOfProgeny` |
| `search_by_lpn(lpn_id)` | -- | Concurrent: `animal_details` + `lineage` + `progeny` |

---

## See Also

- [Library API Reference](LIBRARY-API.md) -- full method signatures
- [CLI Reference](CLI.md) -- all command-line options
- [Error Handling Reference](ERROR-HANDLING.md) -- error types and retry behavior
- [How to Configure Timeout and Retries](../how-to/CONFIGURE-CLIENT.md)

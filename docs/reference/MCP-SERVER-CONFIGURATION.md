---
diataxis_type: reference
---

# MCP Server Configuration Reference

Complete reference for all configuration options of the `nsip mcp` server. This covers transport, tool sets, authentication, telemetry, and HTTP middleware.

For the tool API surface, see [MCP Tools Reference](MCP-TOOLS.md).

---

## Transport Options

The `nsip mcp` subcommand accepts the following transport flags:

| Flag | Default | Description |
|------|---------|-------------|
| `--transport` | `stdio` | Transport type: `stdio` or `http` |
| `--host` | `127.0.0.1` | Bind address for the HTTP listener. Ignored when transport is `stdio`. |
| `--port` | `8080` | Listen port for the HTTP listener. Ignored when transport is `stdio`. |

When `--transport stdio` is active, the server reads JSON-RPC messages from stdin and writes responses to stdout. No network socket is opened.

When `--transport http` is active, the server binds to `--host`:`--port` and serves streamable HTTP with SSE.

---

## Tool Set Options

| Flag | Default | Description |
|------|---------|-------------|
| `--tools` | all enabled | Comma-separated list of tool sets to enable: `search`, `analytics`, `flock`, `breed` |

When `--tools` is omitted, all four tool sets are enabled. When provided, only the listed sets are registered with the MCP server.

### Tool Set Mapping

Each tool set maps to one or more MCP tools:

| Tool Set | Tools |
|----------|-------|
| `search` | `search`, `details`, `lineage`, `progeny`, `profile` |
| `analytics` | `compare`, `rank`, `inbreeding_check`, `mating_recommendations` |
| `flock` | `flock_summary`, `database_status` |
| `breed` | `breed_groups`, `trait_ranges` |

### Dynamic Instructions

The server generates `ServerInfo.instructions` text dynamically based on which tool sets are enabled. When tool sets are filtered via `--tools`, only documentation for enabled tools appears in the instructions. The following sections are included:

| Section | Condition |
|---------|-----------|
| Header | Always present |
| Search & Retrieval Tools | `search` enabled |
| Analytics Tools | `analytics` enabled |
| Flock Tools | `flock` enabled |
| Breed Tools | `breed` enabled |
| Resources guide | Always present |
| Guided Prompts guide | Always present |
| Common Parameters | Always present |

---

## Authentication Options

| Flag | Default | Description |
|------|---------|-------------|
| `--auth` | disabled | Enable OAuth 2.1 + PAT bearer authentication. HTTP transport only; ignored for stdio. |

Authentication is only meaningful over HTTP. When `--auth` is set, the server requires a valid bearer token on all MCP requests and exposes the OAuth endpoints listed below.

### OAuth Environment Variables

The first four variables are required when `--auth` is set. The remaining three are optional with sensible defaults:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `NSIP_GITHUB_CLIENT_ID` | Yes | — | GitHub OAuth app client ID |
| `NSIP_GITHUB_CLIENT_SECRET` | Yes | — | GitHub OAuth app client secret |
| `NSIP_AUTH_SECRET` | Yes | — | HMAC-SHA256 secret for JWT signing (minimum 32 characters recommended) |
| `NSIP_AUTH_BASE_URL` | Yes | — | External base URL for redirect URIs (e.g. `http://localhost:8080`) |
| `NSIP_AUTH_ISSUER` | No | value of `NSIP_AUTH_BASE_URL` | JWT `iss` claim value. Override when the public-facing issuer URL differs from the base URL. |
| `NSIP_AUTH_TOKEN_TTL` | No | `3600` | JWT token time-to-live in seconds. |
| `NSIP_AUTH_ALLOWED_USERS` | No | *(none — all users allowed)* | Comma-separated allowlist of GitHub usernames. When set, only listed users may authenticate. |

### OAuth Endpoints

These endpoints are exposed only when `--auth` is set:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/register` | POST | Dynamic client registration (RFC 7591) |
| `/authorize` | GET | Authorization endpoint with PKCE |
| `/callback` | GET | GitHub OAuth callback handler |
| `/token` | POST | Token exchange (authorization code or refresh token) |
| `/.well-known/oauth-authorization-server` | GET | Authorization server metadata (RFC 8414) |
| `/.well-known/oauth-protected-resource` | GET | Protected resource metadata (RFC 9728) |

### PAT Authentication

Personal access tokens are accepted as bearer tokens alongside OAuth JWTs. Token prefix detection determines the validation path:

| Prefix | Source |
|--------|--------|
| `ghp_` | GitHub personal access token (classic) |
| `gho_` | GitHub OAuth token |
| `github_pat_` | GitHub fine-grained personal access token |

Tokens matching these prefixes are validated against the GitHub API. Validated PATs are cached in-memory with a 5-minute TTL.

### JWT Configuration

JWTs issued by the OAuth flow use the following parameters:

| Parameter | Value |
|-----------|-------|
| Algorithm | HMAC-SHA256 |
| Audience (`aud`) | `nsip-mcp` |
| Default TTL | 3600 seconds (1 hour) |

The signing key is derived from `NSIP_AUTH_SECRET`.

---

## Telemetry

Telemetry is a compile-time feature, not a runtime flag.

| Feature Flag | Build Command |
|--------------|---------------|
| `telemetry` | `cargo build --features telemetry` |

| Feature State | Log Format | Output |
|---------------|------------|--------|
| Enabled | JSON with W3C trace context (`trace_id`, `span_id`) | stderr |
| Disabled (default) | Plain text tracing | stderr |

---

## HTTP Middleware Stack

When `--transport http` is active, the following middleware layers are applied to every request, in order from outermost to innermost:

| Order | Middleware | Behavior |
|-------|-----------|----------|
| 1 | SSE Accept header normalization | Ensures SSE requests carry the correct `Accept` header |
| 2 | Request/response logging | Logs method, path, status, and latency via tracing |
| 3 | Origin header validation | DNS rebinding protection; restricts to localhost origins |
| 4 | CORS | Allows any origin; exposes `mcp-session-id` header |
| 5 | Session status correction | Rewrites rmcp 401 responses to 404 |
| 6 | Bearer auth | Token validation. Present only when `--auth` is enabled. |

---

## See Also

- [CLI Reference](CLI.md) -- global options and all subcommands
- [MCP Server API](../MCP.md) -- tools, resources, and prompts
- [How to Enable OAuth](../how-to/OAUTH-AUTHENTICATION.md)
- [How to Configure Tool Sets](../how-to/MCP-TOOL-SETS.md)

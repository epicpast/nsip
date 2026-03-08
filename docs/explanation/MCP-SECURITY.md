# MCP Server Security Model

> This document explains the design decisions, trade-offs, and threat model behind the NSIP MCP server's authentication and transport security. It is understanding-oriented -- read this to learn *why* the security layer works the way it does, not *how* to configure it.

---

## Why Authenticate an MCP Server?

The MCP protocol supports two transports: stdio and HTTP. Over stdio, the MCP server runs as a child process of the client. The client spawns it, owns its stdin/stdout, and controls its lifecycle. There is no network surface, so there is nothing to authenticate.

HTTP transport changes the picture entirely. When the server binds to a network address, anyone who can reach that address can send JSON-RPC requests. Without authentication, a third party could query the NSIP Search API through your server, consume your rate limits, and access data you intended to keep behind a controlled interface.

The NSIP MCP server therefore requires bearer token authentication on the `/mcp` endpoint when running in HTTP mode. OAuth protocol endpoints (registration, authorization, callback, token exchange, and discovery metadata) remain unauthenticated because they are part of the authentication flow itself.

When OAuth is not configured (no environment variables set), the server runs without authentication. This is intentional for local development and stdio transport, where the process boundary provides equivalent isolation.

---

## OAuth 2.1 with PKCE

The server implements OAuth 2.1 with Proof Key for Code Exchange (PKCE) using the S256 challenge method. OAuth 2.1 is a strict profile of OAuth 2.0 that mandates PKCE for all authorization code grants, removing the implicit and resource owner password grants entirely.

### Why PKCE Matters

In a standard authorization code flow without PKCE, an attacker who intercepts the authorization code (via a malicious app registered with the same redirect URI scheme, or by observing the redirect) can exchange it for an access token. The authorization code alone is sufficient.

PKCE closes this gap. Before initiating the flow, the client generates a random `code_verifier` and derives a `code_challenge` from it using SHA-256. The challenge is sent with the authorization request; the verifier is sent with the token exchange. The server recomputes the challenge from the verifier and compares. An attacker who intercepts only the authorization code cannot complete the exchange without the verifier, which never leaves the client.

The NSIP server enforces S256 exclusively. Plain PKCE (where the challenge equals the verifier) is rejected because it provides no protection against interception.

### The Authorization Flow

The full flow proceeds as follows:

1. **Dynamic client registration.** The MCP client registers itself at `POST /register` with its redirect URIs. The server assigns a `client_id`. This follows RFC 7591 and allows new clients to onboard without manual configuration.

2. **Authorization request.** The client redirects the user to `GET /authorize` with its `client_id`, `redirect_uri`, `code_challenge`, and `state`. The server validates the client registration, stores a pending authorization keyed by an internal state token, and redirects the user to GitHub.

3. **GitHub authentication.** The user authenticates with GitHub. GitHub redirects back to the server's `/callback` endpoint with an authorization code and the internal state token.

4. **Callback processing.** The server exchanges the GitHub authorization code for a GitHub access token, fetches the user's GitHub profile, and issues its own authorization code back to the MCP client via the client's redirect URI.

5. **Token exchange.** The client exchanges the authorization code plus the `code_verifier` at `POST /token`. The server verifies the PKCE challenge, validates the code, and returns a JWT access token and an opaque refresh token.

The PKCE verification uses constant-time comparison (`subtle::ConstantTimeEq`) to prevent timing side-channel attacks on the challenge value.

---

## GitHub as Identity Provider

The NSIP MCP server uses GitHub as its sole identity provider. This is a deliberate choice, not a default.

The primary consumers of the MCP server are developer-facing AI assistants: Claude Code, Cursor, Windsurf, and similar tools. The people running these tools are developers who overwhelmingly have GitHub accounts. Adding a generic OIDC provider or a local user database would increase complexity without serving the actual audience.

GitHub provides two things the server needs: a trusted identity (the GitHub login) and a consent screen the user already understands. The server requests only the `read:user` scope, which grants access to the user's public profile. No repository access, no organization data, no write permissions.

After authenticating the user through GitHub, the server issues its own JWT. The GitHub token is used only during the callback to fetch the user's login and is not stored. Subsequent requests to the MCP server use the NSIP-issued JWT, not the GitHub token.

An optional `NSIP_AUTH_ALLOWED_USERS` environment variable provides an allowlist of GitHub usernames. When set, only listed users can authenticate. When unset, any GitHub user can obtain a token. This gives operators simple access control without requiring a full authorization layer.

---

## GitHub PAT Shortcut

The full OAuth 2.1 flow with PKCE involves multiple redirects, a browser interaction, and dynamic client registration. For simpler integrations -- scripts, CI pipelines, testing -- this overhead is unnecessary.

The server offers a shortcut: pass a GitHub Personal Access Token (PAT) directly as a `Bearer` token. The middleware detects PAT prefixes (`ghp_`, `gho_`, `github_pat_`) and validates the token by calling the GitHub API (`GET /user`) instead of performing JWT validation.

### Caching and Performance

Validating a PAT requires an HTTP round-trip to `api.github.com` on every request. To avoid this cost, the server caches successful validations in memory with a 5-minute TTL. The cache maps the PAT value to the validated GitHub login and a timestamp. Entries older than 5 minutes are evicted on the next lookup.

The 5-minute TTL balances two concerns:

- **Responsiveness to revocation.** If a PAT is revoked on GitHub, the server continues accepting it for at most 5 minutes. Shorter TTLs reduce this window but increase GitHub API traffic.
- **Performance.** Most MCP sessions involve bursts of tool calls. A 5-minute cache means the first call in a burst hits GitHub; subsequent calls within the session are local lookups.

The cache is an in-memory `HashMap` behind a `RwLock`. It is not bounded by size. In practice this is acceptable because each entry is a short string pair and the number of distinct PATs in a single server instance is small.

### Security Considerations

PATs bypass the PKCE flow entirely. They rely on GitHub's authentication infrastructure rather than the server's own token issuance. This means:

- The server never sees the user's GitHub password. The PAT is a scoped credential issued by GitHub.
- PAT scope is controlled by the user on GitHub. The server only calls `GET /user`, so a fine-grained PAT with no permissions beyond reading the user's own profile is sufficient.
- The PAT is stored in the cache in cleartext. This is acceptable for an in-memory cache in a single-process server, but operators should be aware that the PAT value is held in memory for the cache duration.

---

## JWT Token Design

Access tokens issued by the server are JSON Web Tokens signed with HMAC-SHA256. The design prioritizes simplicity over flexibility.

### Claims

Each JWT contains:

| Claim | Value | Purpose |
|-------|-------|---------|
| `sub` | GitHub login | Identifies the authenticated user |
| `iss` | Configurable (default `https://localhost`) | Identifies the issuing server |
| `aud` | `nsip-mcp` | Restricts the token to this service |
| `exp` | Issue time + TTL | Enforces token expiration |
| `iat` | Current timestamp | Records when the token was issued |
| `jti` | UUIDv4 | Unique identifier for each token |

### Why HMAC-SHA256

Asymmetric algorithms (RS256, ES256) allow third parties to verify tokens without knowing the signing key. This matters in distributed systems where multiple services need to validate tokens independently.

The NSIP MCP server is a single process. The same process signs and validates every token. There is no second service that needs to verify tokens, so asymmetric cryptography adds key management overhead (generating, rotating, and distributing public keys) without benefit. HMAC-SHA256 requires a single shared secret, configured via `NSIP_AUTH_SECRET`.

### Token Lifetimes

Access tokens have a configurable TTL, defaulting to 1 hour (`NSIP_AUTH_TOKEN_TTL`). After expiry, the client must use a refresh token to obtain a new access token.

Refresh tokens are opaque 32-byte random values encoded as base64url strings. They have a 24-hour TTL and are one-time-use: exchanging a refresh token consumes it and issues a new one. This rotation pattern means a stolen refresh token can be used at most once. If the legitimate client and the attacker both try to use the same refresh token, one of them fails, which signals the compromise.

The in-memory store prunes expired entries (pending authorizations, authorization codes, refresh tokens) on each access operation. There is no background reaper thread.

---

## DNS Rebinding Protection

The HTTP transport validates the `Origin` header on every request. When present, the `Origin` must resolve to a localhost address: `localhost`, `127.0.0.1`, `[::1]`, `::1`, or `0.0.0.0`. Requests with a non-local `Origin` are rejected with HTTP 403.

### The Threat

DNS rebinding attacks exploit the browser's same-origin policy. An attacker hosts a page at `evil.example.com` that resolves to a public IP. After the page loads, the attacker changes the DNS record to point `evil.example.com` at `127.0.0.1`. The browser now believes requests to `evil.example.com` are going to a different origin than the MCP server on localhost, but the packets actually arrive at the server. If the server does not validate the `Origin`, it processes the request as if it were legitimate.

By rejecting non-local `Origin` headers, the server ensures that only requests originating from localhost web pages (or requests without an `Origin` header, such as those from non-browser clients) reach the MCP endpoint.

### Limitations

The `Origin` header is browser-enforced. Non-browser clients (curl, scripts, MCP clients) typically do not send it. The server does not require the header -- it only rejects requests where the header is present and non-local. This means DNS rebinding protection applies only to browser-based attack vectors, which is the correct scope: non-browser clients that can set arbitrary headers could also just connect directly.

---

## Stdio Transport Security

The stdio transport has no authentication layer. The server process communicates over stdin/stdout with its parent process (the MCP client). The parent already controls the child's lifecycle, can read its memory, and can terminate it at will. Authentication in this context would protect against nothing.

This is consistent with the Unix process model: if you can fork/exec a process, you own it. The security boundary is the user account that runs the MCP client, not the transport between client and server.

---

## Further Reading

- [OAuth Authentication Setup](../how-to/OAUTH-AUTHENTICATION.md) -- step-by-step configuration instructions
- [MCP Server Configuration Reference](../reference/MCP-SERVER-CONFIGURATION.md) -- environment variables and defaults
- [MCP Tools Reference](../reference/MCP-TOOLS.md) -- the 13 tools exposed by the MCP server

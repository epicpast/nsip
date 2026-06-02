---
diataxis_type: how-to
---
# How to Authenticate with the NSIP MCP Server

> **Problem:** You want to secure the NSIP MCP HTTP server so that only authorized users can access it, using OAuth 2.1 with GitHub as the identity provider or a GitHub Personal Access Token.

**Prerequisites:**
- `nsip` binary installed ([installation options](../MCP.md#installation))
- A GitHub account
- For OAuth flow: a registered GitHub OAuth App (see below)
- For PAT shortcut: a GitHub Personal Access Token

---

## How to Enable OAuth on the MCP HTTP Server

1. Set four required environment variables:

   ```bash
   export NSIP_GITHUB_CLIENT_ID="your-github-oauth-client-id"
   export NSIP_GITHUB_CLIENT_SECRET="your-github-oauth-client-secret"
   export NSIP_AUTH_SECRET="a-strong-random-string-for-signing-jwts"
   export NSIP_AUTH_BASE_URL="https://your-server.example.com"
   ```

2. Start the server with the `--auth` flag:

   ```bash
   nsip mcp --transport http --port 8080 --auth
   ```

   The `--auth` flag only applies to the HTTP transport. It has no effect on stdio transport.

3. Verify the server is enforcing authentication by sending an unauthenticated JSON-RPC request to the `/mcp` endpoint:

   ```bash
   curl -s -X POST http://localhost:8080/mcp \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
   ```

   You should receive a `401 Unauthorized` response with a `WWW-Authenticate` header.

**Optional environment variables:**

| Variable | Default | Description |
|---|---|---|
| `NSIP_AUTH_ISSUER` | `https://localhost` | JWT issuer claim |
| `NSIP_AUTH_TOKEN_TTL` | `3600` | Access token lifetime in seconds |
| `NSIP_AUTH_ALLOWED_USERS` | *(none)* | Comma-separated allowlist of GitHub usernames |

---

## How to Create a GitHub OAuth App

1. Go to **GitHub Settings > Developer settings > OAuth Apps**.
2. Click **New OAuth App**.
3. Fill in the fields:
   - **Application name:** Choose any name (e.g., "NSIP MCP Server").
   - **Homepage URL:** Your server URL (e.g., `https://your-server.example.com`).
   - **Authorization callback URL:** Set this to `{NSIP_AUTH_BASE_URL}/callback` (e.g., `https://your-server.example.com/callback`).
4. Click **Register application**.
5. Copy the **Client ID** into `NSIP_GITHUB_CLIENT_ID`.
6. Generate a new **Client Secret** and copy it into `NSIP_GITHUB_CLIENT_SECRET`.

---

## How to Authenticate with a GitHub PAT

If you already have a GitHub Personal Access Token, you can skip the full OAuth flow entirely.

1. Pass the token in the `Authorization` header on a JSON-RPC POST to `/mcp`:

   ```bash
   curl -X POST http://localhost:8080/mcp \
     -H "Authorization: Bearer ghp_yourTokenHere" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
   ```

2. The server auto-detects PAT tokens by their prefix (`ghp_`, `gho_`, or `github_pat_`) and validates them against the GitHub API.

3. Successful validations are cached for 5 minutes. Subsequent requests with the same PAT skip the GitHub API call until the cache entry expires.

> **Note:** If `NSIP_AUTH_ALLOWED_USERS` is set, the GitHub username associated with the PAT must appear in that allowlist.

---

## How to Authenticate via the Full OAuth Flow

Use this when building a client that authenticates end-users through the browser.

### Step 1: Register your client dynamically

Send a POST request to `/register`:

```bash
curl -X POST http://localhost:8080/register \
  -H "Content-Type: application/json" \
  -d '{
    "redirect_uris": ["https://your-app.example.com/callback"],
    "client_name": "My MCP Client",
    "grant_types": ["authorization_code", "refresh_token"]
  }'
```

The response contains your `client_id`:

```json
{
  "client_id": "generated-uuid",
  "redirect_uris": ["https://your-app.example.com/callback"]
}
```

### Step 2: Generate a PKCE code verifier and challenge

```bash
CODE_VERIFIER=$(openssl rand -base64 32 | tr -d '=' | tr '/+' '_-')
CODE_CHALLENGE=$(printf '%s' "$CODE_VERIFIER" | openssl dgst -sha256 -binary | openssl base64 | tr -d '=' | tr '/+' '_-')
```

### Step 3: Redirect the user to the authorization endpoint

Open this URL in the user's browser:

```
http://localhost:8080/authorize?\
  client_id={client_id}&\
  response_type=code&\
  redirect_uri=https://your-app.example.com/callback&\
  code_challenge={CODE_CHALLENGE}&\
  code_challenge_method=S256&\
  state={random_state_value}
```

All parameters are required:

| Parameter | Value |
|---|---|
| `client_id` | From the registration response |
| `response_type` | Must be `code` |
| `redirect_uri` | Must match one of the registered URIs |
| `code_challenge` | Base64url-encoded SHA-256 hash of the code verifier |
| `code_challenge_method` | Must be `S256` |
| `state` | Random string to prevent CSRF |

The server redirects the user to GitHub for login, then back to `/callback`, which redirects to your `redirect_uri` with `code` and `state` query parameters.

### Step 4: Exchange the authorization code for tokens

```bash
curl -X POST http://localhost:8080/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code\
&code={authorization_code}\
&redirect_uri=https://your-app.example.com/callback\
&code_verifier={CODE_VERIFIER}\
&client_id={client_id}"
```

The response contains your access and refresh tokens:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "bearer",
  "expires_in": 3600,
  "refresh_token": "opaque-refresh-token-string"
}
```

### Step 5: Use the access token

Include it in the `Authorization` header on JSON-RPC POSTs to `/mcp`:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..." \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

---

## How to Use a Refresh Token

When the access token expires, exchange the refresh token for a new token pair.

1. Send a POST request to `/token` with `grant_type=refresh_token`:

   ```bash
   curl -X POST http://localhost:8080/token \
     -H "Content-Type: application/x-www-form-urlencoded" \
     -d "grant_type=refresh_token\
   &refresh_token={your_refresh_token}\
   &client_id={client_id}"
   ```

2. The response contains a new access token and a new refresh token:

   ```json
   {
     "access_token": "eyJhbGciOiJIUzI1NiIs...",
     "token_type": "bearer",
     "expires_in": 3600,
     "refresh_token": "new-opaque-refresh-token"
   }
   ```

3. Store the new refresh token. The old refresh token is consumed and cannot be reused. Each refresh produces a rotated token pair.

---

## How to Discover OAuth Endpoints

Use the standard discovery documents to find all OAuth endpoints programmatically.

1. Fetch authorization server metadata:

   ```bash
   curl -s http://localhost:8080/.well-known/oauth-authorization-server | jq .
   ```

   ```json
   {
     "issuer": "https://localhost",
     "authorization_endpoint": "https://your-server.example.com/authorize",
     "token_endpoint": "https://your-server.example.com/token",
     "registration_endpoint": "https://your-server.example.com/register",
     "response_types_supported": ["code"],
     "grant_types_supported": ["authorization_code", "refresh_token"],
     "token_endpoint_auth_methods_supported": ["none"],
     "code_challenge_methods_supported": ["S256"]
   }
   ```

2. Fetch protected resource metadata:

   ```bash
   curl -s http://localhost:8080/.well-known/oauth-protected-resource | jq .
   ```

   ```json
   {
     "resource": "https://your-server.example.com",
     "authorization_servers": ["https://your-server.example.com"],
     "bearer_methods_supported": ["header"]
   }
   ```

Both discovery endpoints are accessible without authentication.

---

## See Also

- [MCP Security (Explanation)](../explanation/MCP-SECURITY.md) -- background on the security model, threat model, and design decisions
- [MCP Server Configuration (Reference)](../reference/MCP-SERVER-CONFIGURATION.md) -- full environment variable reference and transport options

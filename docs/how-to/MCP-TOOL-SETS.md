---
diataxis_type: how-to
---

# How to Filter MCP Tool Sets

> **Problem:** You want to expose only a subset of the 13 NSIP MCP tools to your client, either to reduce noise for a focused workflow or to limit what an untrusted client can invoke.

**Prerequisites:**

- `nsip` binary installed ([installation options](../MCP.md#installation))
- An MCP-compatible client or the ability to send raw JSON-RPC requests

---

## Expose only specific tool sets

Pass `--tools` with a comma-separated list of set names. No spaces between set names.

```bash
nsip mcp --tools search,breed
```

This starts the server with only the `search` and `breed` tools. Tools from `analytics` and `flock` are not registered and will not appear in `tools/list` responses.

To expose a single set:

```bash
nsip mcp --tools analytics
```

To expose all sets, omit `--tools` entirely:

```bash
nsip mcp
```

---

## Check which tools a set includes

Each set name maps to a fixed group of tools:

| Set | Tools included |
|-----|----------------|
| `search` | `search`, `details`, `lineage`, `progeny`, `profile` |
| `analytics` | `compare`, `rank`, `inbreeding_check`, `mating_recommendations` |
| `flock` | `flock_summary`, `database_status` |
| `breed` | `breed_groups`, `trait_ranges` |

All four sets together total 13 tools. The sets are mutually exclusive -- each tool belongs to exactly one set.

---

## Use tool sets with HTTP transport

Combine `--tools` with `--transport http` and any other HTTP flags:

```bash
nsip mcp --transport http --port 8080 --tools search
```

This serves only the five `search` tools over HTTP at `http://127.0.0.1:8080/mcp`. The `--host`, `--port`, and `--auth` flags work the same as without `--tools`.

To expose multiple sets over HTTP:

```bash
nsip mcp --transport http --host 0.0.0.0 --port 9090 --tools search,analytics
```

---

## Use tool sets with stdio transport

Stdio is the default transport. Pass `--tools` directly:

```bash
nsip mcp --tools search,breed
```

In an MCP client configuration file (`.mcp.json` or `claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "nsip": {
      "command": "nsip",
      "args": ["mcp", "--tools", "search,breed"]
    }
  }
}
```

---

## Verify which tools are exposed

After starting the server, send a `tools/list` JSON-RPC request to confirm the active tools.

### Stdio

Pipe an `initialize` handshake followed by `tools/list`:

```bash
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}\n' \
  | nsip mcp --tools search
```

The response with `"id":2` contains a `tools` array listing only the tools from enabled sets. With `--tools search`, you see five entries: `search`, `details`, `lineage`, `progeny`, `profile`.

### HTTP

```bash
# Start the server in the background
nsip mcp --transport http --port 8080 --tools flock &

# Send tools/list (after initializing a session)
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'

# Use the Mcp-Session-Id from the response header
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <session-id>" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
```

With `--tools flock`, the `tools` array contains exactly two entries: `flock_summary` and `database_status`.

---

## Understand how server instructions adapt

When you filter tool sets, the `ServerInfo.instructions` field returned during initialization changes automatically. The server omits documentation sections for disabled tool sets, so MCP clients receive instructions that describe only the tools they can actually call.

For example, `--tools search` produces instructions covering search, details, lineage, progeny, and profile -- with no mention of analytics, flock, or breed tools. This keeps client-side context focused and avoids confusion about unavailable tools.

No configuration is needed for this behavior. It happens whenever `--tools` is present.

---

## See also

- [MCP Server Configuration Reference](../reference/MCP-SERVER-CONFIGURATION.md) -- full flag reference for `nsip mcp`
- [Dynamic Instructions](../explanation/DYNAMIC-INSTRUCTIONS.md) -- how and why instructions adapt to enabled tool sets
- [How to Use MCP Tools](USE-MCP-TOOLS.md) -- general MCP server setup and usage

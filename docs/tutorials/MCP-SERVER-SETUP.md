---
diataxis_type: tutorial
---
# Setting Up the NSIP MCP Server

> **Learning Goal:** By the end of this tutorial, you will have the NSIP MCP server running and connected to an AI assistant (Claude Desktop or Claude Code), ready to query sheep genetic data through natural language.

**Time to complete:** 10 minutes
**Prerequisites:** One of the following AI clients installed:
- [Claude Desktop](https://claude.ai/download)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code)

---

## What You Will Build

A working MCP (Model Context Protocol) integration that lets your AI assistant:

1. Search the NSIP sheep genetics database
2. Look up individual animal profiles
3. Compare animals by their genetic traits
4. Access breed group and trait reference data

---

## Step 1: Install the NSIP Binary

The MCP server is built into the `nsip` command-line tool. Choose one installation method:

**From crates.io (requires Rust 1.92+):**

```bash
cargo install nsip
```

**From pre-built binaries:**

Download the binary for your platform from [GitHub Releases](https://github.com/zircote/nsip/releases):

| Platform       | Binary                        |
|----------------|-------------------------------|
| Linux x86_64   | `nsip-0.6.0-linux-amd64`       |
| Linux ARM64    | `nsip-0.6.0-linux-arm64`       |
| macOS x86_64   | `nsip-0.6.0-macos-amd64`       |
| macOS ARM64    | `nsip-0.6.0-macos-arm64`       |
| Windows x86_64 | `nsip-0.6.0-windows-amd64.exe` |

After downloading, make it executable and move it to your PATH:

```bash
chmod +x nsip-0.6.0-macos-arm64
sudo mv nsip-0.6.0-macos-arm64 /usr/local/bin/nsip
```

**Via Docker:**

```bash
docker pull ghcr.io/zircote/nsip
```

**Verify the installation:**

```bash
nsip --version
```

---

## Step 2: Test the MCP Server Locally

Before connecting to an AI client, verify that the MCP server starts correctly:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | nsip mcp
```

You should see a JSON response containing the server's capabilities (tools, resources, and prompts). Press Ctrl+C to stop.

**What just happened?** The `nsip mcp` command starts a stdio-based MCP server. It reads JSON-RPC messages from stdin and writes responses to stdout. The `initialize` message is the first step of the MCP handshake.

---

## Step 3: Configure Your AI Client

Choose the client you want to use:

### Option A: Claude Code

Create a `.mcp.json` file in your project root (or at `~/.mcp.json` for global access):

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

Restart Claude Code or open a new session. The NSIP tools will be available automatically.

### Option B: Claude Desktop

Open the Claude Desktop configuration file:

- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux:** `~/.config/Claude/claude_desktop_config.json`

Add the NSIP server to the `mcpServers` section:

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

Restart Claude Desktop. You should see the NSIP tools listed in the tools panel.

### Option C: Docker Transport

If you installed via Docker, use this configuration instead (works with both Claude Code and Claude Desktop):

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

**What just happened?** The configuration tells your AI client how to launch the NSIP MCP server. When the client starts, it spawns the `nsip mcp` process and communicates with it over stdio using the MCP protocol.

---

## Step 4: Verify the Connection

In your AI client, try asking a question that uses the NSIP tools:

```
What breed groups are available in the NSIP database?
```

The AI assistant should use the `breed_groups` tool and return a list of breed groups with their breeds.

Try a few more queries:

```
Search for current female animals in breed 640
```

```
Look up the profile for animal 400001
```

```
When was the NSIP database last updated?
```

**What just happened?** The AI client matched your natural language query to one of the 13 NSIP MCP tools and called it automatically. The server fetched the data from the NSIP Search API and returned it to the client for display.

---

## Step 5: Explore Available Tools

The NSIP MCP server provides 13 tools:

| Tool | Description |
|------|-------------|
| `search` | Search for animals with filters (breed, gender, status, date range, flock) |
| `details` | Get detailed EBV data, breed, contact info, and status for an animal |
| `lineage` | Get pedigree / ancestry tree including parents and grandparents |
| `progeny` | Get paginated list of offspring for an animal |
| `profile` | Get complete profile (details + lineage + progeny) in one call |
| `compare` | Compare 2–5 animals side-by-side on their EBV traits |
| `rank` | Rank animals within a breed by weighted EBV trait scores |
| `inbreeding_check` | Calculate Wright's coefficient of inbreeding for a sire-dam pairing |
| `mating_recommendations` | Find optimal mates ranked by trait complementarity and low COI |
| `flock_summary` | Summarize a flock's animals: count, gender breakdown, and average EBVs |
| `database_status` | Get last-updated date and available animal statuses |
| `breed_groups` | List all breed groups and individual breeds |
| `trait_ranges` | Get min/max EBV trait value ranges for a specific breed |

The server also provides 7 guided prompts that help structure common queries. Ask your AI assistant to list the available prompts for more details.

**What just happened?** Each MCP tool maps to one or more NSIP API endpoints. The server handles parameter validation, API calls, and response formatting so the AI client receives clean, structured data.

---

## What You Learned

In this tutorial you:

- Installed the `nsip` binary which includes the MCP server
- Tested the MCP server locally with a raw JSON-RPC message
- Configured Claude Desktop or Claude Code to use the NSIP MCP server
- Verified the connection by querying breed groups and animal data
- Explored the 13 available MCP tools

---

## Going Further

The MCP server has additional features for production deployments:

- **Tool sets** — expose only specific tool categories with `--tools search,breed`. See [How to Configure Tool Sets](../how-to/MCP-TOOL-SETS.md).
- **HTTP transport** — serve over HTTP with `--transport http --port 8080` for network-accessible deployments.
- **OAuth authentication** — secure the HTTP transport with GitHub OAuth via `--auth`. See [How to Enable OAuth](../how-to/OAUTH-AUTHENTICATION.md).
- **Telemetry** — enable W3C trace context in logs with `--features telemetry`. See [How to Enable Telemetry](../how-to/TELEMETRY.md).
- **Dynamic instructions** — the server generates instructions dynamically based on enabled tool sets, so MCP clients only see documentation for available tools. See [Dynamic Instructions](../explanation/DYNAMIC-INSTRUCTIONS.md).

---

## Next Steps

Now that your MCP server is running:

- [Getting Started](GETTING-STARTED.md) -- use the NSIP library directly in Rust code
- [Interpreting Results](INTERPRETING-RESULTS.md) -- understand the genetic data returned by queries
- [Understanding EBVs](../explanation/EBV-EXPLAINED.md) -- background on Estimated Breeding Values
- [MCP Server Configuration](../reference/MCP-SERVER-CONFIGURATION.md) -- full configuration reference

For the complete MCP API reference, see the [MCP Server Reference](../MCP.md).

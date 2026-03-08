# Dynamic MCP Server Instructions

> MCP servers describe themselves to clients through an `instructions` field. The NSIP server generates these instructions dynamically so that clients only learn about capabilities they can actually use.

---

## What are MCP server instructions?

The Model Context Protocol (MCP) defines a handshake between servers and clients. During initialization, the server returns a `ServerInfo` structure that includes an `instructions` field -- a free-form text block that tells the connecting client what the server does, what tools it exposes, what resources are available, and what guided prompts it offers.

Think of this field as a system prompt written by the server for the AI assistant. The client reads these instructions and uses them to decide which tools to call, how to construct resource URIs, and what workflows to suggest to the user. Without instructions, the client has only raw tool schemas -- names, parameter types, and descriptions -- with no higher-level guidance about how the pieces fit together.

The NSIP server uses its instructions to convey:

- **Server identity**: that this is a sheep genetic evaluation server backed by the NSIP Search API.
- **Tool capabilities**: what each of the 13 tools does, grouped by functional area (search, analytics, flock, breed).
- **Resource URIs**: the `nsip://` URI scheme and how to construct URIs for static and templated resources.
- **Guided prompts**: the seven prompts available for structured breeding evaluation workflows.
- **Common parameters**: pagination conventions and output format details.

A client that reads these instructions can operate the server without any external documentation.

---

## Why dynamic generation matters

The NSIP server supports a `--tools` flag that selectively enables subsets of its 13 tools. A deployment might enable only `search` tools for a read-only exploration interface, or only `analytics,breed` for a decision-support application that does not need direct animal lookup.

This creates a problem for static instructions. If the instructions always described all 13 tools, a client connected to a server exposing only 5 tools would receive documentation for 8 tools it cannot call. This leads to two practical issues:

1. **Confusion**: The client may attempt to call tools that do not exist in the current session, producing errors and degrading the user experience.
2. **Wasted tokens**: Every token spent describing unavailable tools is a token that could have been used for the actual conversation. In token-constrained contexts (long conversations, smaller models, cost-sensitive deployments), this overhead is not trivial.

Dynamic instruction generation solves both problems. The server inspects its own configuration at startup and emits instructions that match the tools actually registered. If the analytics tool set is disabled, the "Analytics Tools" section never appears in the instructions text. The client sees a self-consistent view of the server.

---

## How NSIP generates instructions

The `build_instructions()` function in `crates/mcp/instructions.rs` assembles the instructions string from modular sections. It takes a reference to `EnabledToolSets` -- the resolved set of tool groups active for the current server instance -- and conditionally includes each tool section.

The assembly follows a fixed order:

1. **Header** -- always included. Identifies the server as the "NSIP Livestock Intelligence Server" and states the overall scope (sheep genetic evaluation, breeding decision support, flock analytics).

2. **Tool sections** -- conditionally included based on `EnabledToolSets`:
   - **Search & Retrieval Tools** (5 tools): `search`, `details`, `lineage`, `progeny`, `profile`. Included when the `Search` set is enabled.
   - **Analytics Tools** (4 tools): `compare`, `rank`, `inbreeding_check`, `mating_recommendations`. Included when the `Analytics` set is enabled.
   - **Flock Tools** (2 tools): `flock_summary`, `database_status`. Included when the `Flock` set is enabled.
   - **Breed Tools** (2 tools): `breed_groups`, `trait_ranges`. Included when the `Breed` set is enabled.

3. **Resource guide** -- always included. Documents the `nsip://` URI scheme regardless of which tools are active, because resources are independent of tools.

4. **Guided prompts** -- always included. Lists all seven prompts with descriptions. Prompts are not filtered by tool sets because they orchestrate multi-step workflows that may reference tools the user can still benefit from understanding.

5. **Common parameters appendix** -- always included. Documents pagination and output format conventions.

The function pre-allocates a 4 KiB buffer and builds the output through `push_str` calls, avoiding repeated allocations. The result is a single `String` assigned directly to the `ServerInfo.instructions` field during server initialization.

When all four tool sets are enabled (the default), the instructions contain all sections and describe all 13 tools. When tool sets are selectively disabled, the corresponding sections disappear. Even with every tool set disabled, the header, resource guide, prompt guide, and common parameters remain -- the server still has resources and prompts to describe.

---

## The `nsip://` URI scheme

MCP resources use URIs to identify retrievable data. The NSIP server defines a custom `nsip://` scheme rather than using `https://` URLs for its resources. This is a deliberate choice: resources represent processed, structured data from the NSIP API, not raw HTTP endpoints.

The scheme divides into two categories:

**Static resources** have fixed URIs that never change:

| URI | Content |
|---|---|
| `nsip://glossary` | EBV trait glossary with descriptions and units |
| `nsip://breeds` | Complete breed listing with IDs |
| `nsip://guide/selection` | Selection strategy guide |
| `nsip://guide/inbreeding` | Inbreeding management reference |
| `nsip://status` | Database status and last-updated timestamp |

**Resource templates** contain placeholders that the client fills in:

| URI Template | Content |
|---|---|
| `nsip://animal/{lpn_id}` | Full animal profile by LPN ID |
| `nsip://animal/{lpn_id}/pedigree` | Pedigree/lineage tree |
| `nsip://animal/{lpn_id}/progeny` | Offspring listing |
| `nsip://breed/{breed_id}/ranges` | Breed EBV percentile ranges |

The instructions document these URIs so that clients can construct resource requests without trial and error. A client that needs breed percentile ranges for breed 640 knows to request `nsip://breed/640/ranges` directly.

---

## Design principle: self-describing servers

The NSIP server's instructions follow a "describe what is available" principle. A client should be able to understand the complete server API -- tools, resources, prompts, and conventions -- from the instructions alone, without consulting external documentation.

This aligns with the MCP specification's intent for the `instructions` field. The protocol designers included this field precisely so that servers could be self-describing. An AI assistant connecting to an unknown MCP server should be able to read the instructions and immediately begin using the server effectively.

For the NSIP server, this means the instructions are the authoritative source of truth for any connected client. The external documentation (including this document) explains the design rationale and implementation details. The instructions themselves are operational -- they tell the client what to do.

Dynamic generation ensures this self-description remains accurate. When the server configuration changes, the instructions change with it. There is no synchronization problem between a static instructions file and the server's actual capabilities.

---

## Related documentation

- [Using MCP Tool Sets](../how-to/MCP-TOOL-SETS.md) -- how to enable and disable tool sets with the `--tools` flag
- [MCP Server Configuration Reference](../reference/MCP-SERVER-CONFIGURATION.md) -- complete configuration options for the MCP server

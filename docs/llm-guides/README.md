# LLM Guide Templates for NSIP MCP Server

Ready-to-use instruction files that teach AI assistants how to use the NSIP MCP tools for sheep genetic evaluation.

## Usage

Copy the appropriate file into your project. Each template is self-contained.

| File | Target | Destination |
|---|---|---|
| `CLAUDE.md` | Claude Code / Claude Desktop | Project root (`CLAUDE.md`) or merge into existing |
| `AGENTS.md` | Generic AI agents (OpenAI, Copilot coding agent, etc.) | Project root |
| `copilot-instructions.md` | GitHub Copilot | `.github/copilot-instructions.md` |
| `cursorrules` | Cursor IDE | `.cursorrules` at project root |
| `GEMINI.md` | Google Gemini | Project root or context window |

## What's Included

Each template covers:

- **Server configuration** -- how to set up `.mcp.json` or equivalent
- **Tool quick-reference** -- all 13 NSIP MCP tools with key parameters
- **Common workflows** -- evaluate animal, plan mating, rank flock, flock improvement
- **Data conventions** -- LPN IDs, breed IDs, EBV traits, selection direction
- **Error handling tips** -- not-found responses, pagination, validation

## Example Project

Clone [zircote/nsip-example](https://github.com/zircote/nsip-example) for a working farm repository with MCP config, sample workflows, and AI assistant instructions already set up.

## Full Reference

For complete parameter tables, response schemas, analytics formulas, and the EBV glossary, see [`docs/MCP.md`](../MCP.md).

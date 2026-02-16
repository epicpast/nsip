# CLI Reference

Complete reference for the `nsip` command-line interface.

---

## Synopsis

```
nsip [OPTIONS] <COMMAND>
```

## Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--json` | `-J` | Output raw JSON instead of human-readable format |
| `--version` | `-V` | Print version information |
| `--help` | `-h` | Print help information |

The `--json` flag is global and applies to all subcommands. When set, the output is the raw JSON response from the NSIP API. When omitted (the default), output is formatted as human-readable ASCII tables.

---

## Commands

### date-updated

Get the date when the NSIP database was last updated.

```
nsip date-updated
nsip -J date-updated
```

**Arguments:** None

**Output:** The last-updated date from the NSIP Search API. Always outputs JSON regardless of the `--json` flag.

---

### breed-groups

List all available breed groups and the individual breeds within each group.

```
nsip breed-groups
nsip -J breed-groups
```

**Arguments:** None

**Output (default):** ASCII table of breed groups with their breeds and IDs.

**Output (JSON):** Array of `BreedGroup` objects, each containing an `id`, `name`, and `breeds` array.

---

### statuses

List all available animal statuses.

```
nsip statuses
nsip -J statuses
```

**Arguments:** None

**Output (default):** Bullet list of status strings (e.g., `CURRENT`, `SOLD`, `DEAD`).

**Output (JSON):** Array of status strings.

---

### trait-ranges

Get the minimum and maximum EBV trait values for a specific breed.

```
nsip trait-ranges <BREED_ID>
nsip -J trait-ranges <BREED_ID>
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `BREED_ID` | integer | yes | Breed ID to query trait ranges for |

**Validation:** `breed_id` must be greater than 0.

**Example:**

```bash
nsip trait-ranges 486
nsip -J trait-ranges 640
```

---

### search

Search for animals in the NSIP database with filters for breed, gender, status, date range, flock, and sorting.

```
nsip search [OPTIONS]
```

**Options:**

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--breed-id` | `-b` | integer | -- | Breed ID to filter by |
| `--breed-group-id` | -- | integer | -- | Breed group ID to filter by |
| `--status` | `-s` | string | -- | Animal status filter (`CURRENT`, `SOLD`, `DEAD`) |
| `--gender` | `-g` | string | -- | Gender filter (`Male`, `Female`, `Both`) |
| `--born-after` | -- | string | -- | Only animals born after this date (`YYYY-MM-DD`) |
| `--born-before` | -- | string | -- | Only animals born before this date (`YYYY-MM-DD`) |
| `--proven-only` | -- | flag | false | Only return proven animals |
| `--flock-id` | -- | string | -- | Flock ID to filter by |
| `--sort-by` | -- | string | -- | Trait abbreviation to sort by (e.g., `BWT`, `WWT`) |
| `--reverse` | -- | flag | false | Reverse the sort order |
| `--page` | `-p` | integer | 0 | Page number (0-indexed) |
| `--page-size` | -- | integer | 15 | Results per page (1-100) |

**Validation:** `page_size` must be between 1 and 100.

**Examples:**

```bash
# Search for current male Dorper sheep sorted by weaning weight
nsip search --breed-id 486 --gender Male --status CURRENT --sort-by WWT

# Get page 2 with 25 results per page
nsip search --breed-id 486 --page 2 --page-size 25

# Search with date range and JSON output
nsip -J search --breed-id 640 --born-after 2020-01-01 --born-before 2023-12-31

# Only proven animals from a specific flock
nsip search --breed-id 486 --flock-id 430735 --proven-only
```

---

### details

Get detailed information about a specific animal, including EBV traits, breed, contact info, and status.

```
nsip details <SEARCH_STRING>
nsip -J details <SEARCH_STRING>
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `SEARCH_STRING` | string | yes | LPN ID or registration number of the animal |

**Validation:** `search_string` must not be empty or whitespace-only.

**Examples:**

```bash
nsip details 430735-0032
nsip -J details 430735-0032
```

---

### lineage

Get lineage (ancestry) information for a specific animal, including sire, dam, and extended pedigree.

```
nsip lineage <LPN_ID>
nsip -J lineage <LPN_ID>
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `LPN_ID` | string | yes | LPN ID of the animal |

**Validation:** `lpn_id` must not be empty or whitespace-only.

**Examples:**

```bash
nsip lineage 430735-0032
nsip -J lineage 430735-0032
```

---

### progeny

Get progeny (offspring) information for a specific animal with pagination.

```
nsip progeny [OPTIONS] <LPN_ID>
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `LPN_ID` | string | yes | LPN ID of the animal |

**Options:**

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--page` | `-p` | integer | 0 | Page number (0-indexed) |
| `--page-size` | -- | integer | 10 | Results per page |

**Validation:** `lpn_id` must not be empty. `page_size` must be greater than 0.

**Examples:**

```bash
nsip progeny 430735-0032
nsip progeny 430735-0032 --page 1 --page-size 20
nsip -J progeny 430735-0032
```

---

### profile

Get a full profile for an animal, combining details, lineage, and progeny in a single call. Internally fetches all three concurrently using `tokio::join!`.

```
nsip profile <LPN_ID>
nsip -J profile <LPN_ID>
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `LPN_ID` | string | yes | LPN ID of the animal |

**Validation:** `lpn_id` must not be empty or whitespace-only.

**Examples:**

```bash
nsip profile 430735-0032
nsip -J profile 430735-0032
```

---

### compare

Compare two or more animals side-by-side on their EBV traits. Fetches details for all animals concurrently.

```
nsip compare [OPTIONS] <LPN_IDS>...
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `LPN_IDS` | string (2-5) | yes | LPN IDs of animals to compare |

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--traits` | string | Comma-separated list of traits to display (e.g., `BWT,WWT,YWT`) |

**Validation:** Requires 2 to 5 LPN IDs.

**Examples:**

```bash
# Compare two animals on all traits
nsip compare 430735-0032 430735-0041

# Compare three animals on specific traits
nsip compare 430735-0032 430735-0041 430735-0058 --traits BWT,WWT,YWT,EMD

# JSON output
nsip -J compare 430735-0032 430735-0041
```

---

### completions

Generate shell completions for your shell. Write the output to the appropriate completions directory for your shell.

```
nsip completions <SHELL>
```

**Arguments:**

| Argument | Type | Required | Values |
|----------|------|----------|--------|
| `SHELL` | string | yes | `bash`, `zsh`, `fish`, `powershell` |

**Examples:**

```bash
# Bash
nsip completions bash > ~/.local/share/bash-completion/completions/nsip

# Zsh
nsip completions zsh > ~/.zfunc/_nsip

# Fish
nsip completions fish > ~/.config/fish/completions/nsip.fish

# PowerShell
nsip completions powershell > nsip.ps1
```

---

### man-pages

Generate man pages. Writes the main man page to stdout by default, or generates all man pages (including subcommand pages) to a directory.

```
nsip man-pages [OPTIONS]
```

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--out-dir` | string | Output directory for man pages. If omitted, writes the main page to stdout. |

**Examples:**

```bash
# View main man page
nsip man-pages | man -l -

# Generate all man pages to a directory
nsip man-pages --out-dir ./man/man1

# Install system-wide
sudo nsip man-pages --out-dir /usr/local/share/man/man1
```

---

### mcp

Start the MCP (Model Context Protocol) server for AI assistant integration. Communicates over stdio using JSON-RPC.

```
nsip mcp
```

**Arguments:** None

**Notes:**
- Runs as a long-lived process reading JSON-RPC from stdin and writing to stdout
- Logging goes to stderr
- See [MCP Tools Reference](MCP-TOOLS.md) for the full tool catalog

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (validation, API, connection, timeout, parse, or not found) |

On error, the error message is printed to stderr in the format `Error: {message}`.

---

## Output Modes

The CLI supports two output modes controlled by the global `--json` / `-J` flag:

**Human-readable (default):** Formatted ASCII tables and structured text output designed for terminal use.

**JSON (`--json`):** Raw JSON output from the NSIP API, pretty-printed with indentation. Suitable for piping to `jq` or other JSON-processing tools.

```bash
# Pipe JSON output to jq
nsip -J breed-groups | jq '.[0].breeds'

# Save search results to a file
nsip -J search --breed-id 486 > results.json
```

---

## EBV Trait Abbreviations

These abbreviations are used with `--sort-by` and `--traits` options:

| Abbreviation | Name | Unit |
|--------------|------|------|
| BWT | Birth Weight | lbs |
| WWT | Weaning Weight | lbs |
| PWWT | Post-Weaning Weight | lbs |
| YWT | Yearling Weight | lbs |
| FAT | Fat Depth | mm |
| EMD | Eye Muscle Depth | mm |
| NLB | Number of Lambs Born | lambs |
| NWT | Number of Lambs Weaned | lambs |
| PWT | Pounds Weaned | lbs |
| DAG | Dag Score | score |
| WGR | Wool Growth Rate | g/day |
| WEC | Worm Egg Count | eggs/g |
| FEC | Fecal Egg Count | eggs/g |

---

## See Also

- [Library API Reference](LIBRARY-API.md) -- programmatic access to the same functionality
- [MCP Tools Reference](MCP-TOOLS.md) -- AI assistant integration
- [Configuration Reference](CONFIGURATION.md) -- environment and client configuration
- [Getting Started](../tutorials/GETTING-STARTED.md)

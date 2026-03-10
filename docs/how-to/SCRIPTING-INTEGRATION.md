---
diataxis_type: how-to
---
# How to Integrate NSIP into Scripts and Pipelines

> **Problem:** You want to automate NSIP data retrieval in shell scripts, CI pipelines, or data processing workflows.

**Prerequisites:**
- `nsip` CLI installed and available on `PATH`
- `jq` installed (for JSON processing in shell scripts)

---

## Step 1: Verify CLI Availability

Check that `nsip` is installed and reachable:

```bash
nsip --version
```

In CI pipelines, install from crates.io or download a pre-built binary:

```bash
# From crates.io
cargo install nsip

# Or download a release binary (Linux x86_64 example)
curl -L -o nsip https://github.com/zircote/nsip/releases/latest/download/nsip-linux-amd64
chmod +x nsip
```

---

## Step 2: Use JSON Output in Scripts

All commands accept the `-J` flag for JSON output. This is the recommended mode for scripting because it provides structured, parseable data.

### Extract Specific Fields

```bash
# Get the breed of an animal
nsip details 430735-0032 -J | jq -r '.breed'

# Get all LPN IDs from a search
nsip search --breed-id 486 --status CURRENT -J | jq -r '.results[].lpnId'

# Get the database last-updated date
nsip date-updated -J | jq -r '.'
```

### Check Command Success

```bash
if nsip details 430735-0032 -J > /dev/null 2>&1; then
    echo "Animal found"
else
    echo "Animal not found or API error"
fi
```

The CLI returns a non-zero exit code on errors, which integrates with standard shell error handling.

---

## Step 3: Build a Data Collection Script

Collect details for a list of animals and save as a JSON array:

```bash
#!/usr/bin/env bash
set -euo pipefail

INPUT_FILE="${1:?Usage: $0 <lpn_ids_file>}"
OUTPUT_FILE="${2:-output.json}"

results=()

while IFS= read -r lpn_id; do
    # Skip empty lines and comments
    [[ -z "$lpn_id" || "$lpn_id" == \#* ]] && continue

    if data=$(nsip details "$lpn_id" -J 2>/dev/null); then
        results+=("$data")
    else
        echo "Warning: failed to fetch $lpn_id" >&2
    fi
done < "$INPUT_FILE"

# Combine into a JSON array
printf '%s\n' "${results[@]}" | jq -s '.' > "$OUTPUT_FILE"
echo "Wrote ${#results[@]} records to $OUTPUT_FILE"
```

Usage:

```bash
chmod +x collect_animals.sh
./collect_animals.sh lpn_ids.txt animals.json
```

---

## Step 4: Build a Breed Report Script

Generate a summary report for a breed:

```bash
#!/usr/bin/env bash
set -euo pipefail

BREED_ID="${1:?Usage: $0 <breed_id>}"

echo "=== Breed Report for ID: $BREED_ID ==="

# Get trait ranges
echo "--- Trait Ranges ---"
nsip trait-ranges "$BREED_ID" -J | jq '.'

# Get top animals by WWT
echo "--- Top 10 by Weaning Weight ---"
nsip search --breed-id "$BREED_ID" --status CURRENT \
    --sort-by WWT --reverse --page-size 10 -J | \
    jq -r '.results[] | "\(.lpnId)\t\(.wwt // "N/A")"'
```

---

## Step 5: Use in CI/CD Pipelines

### GitHub Actions Example

```yaml
jobs:
  nsip-report:
    runs-on: ubuntu-latest
    steps:
      - name: Install nsip
        run: cargo install nsip

      - name: Check database freshness
        run: |
          last_updated=$(nsip date-updated -J | jq -r '.')
          echo "NSIP database last updated: $last_updated"

      - name: Generate breed report
        run: |
          nsip search --breed-id 486 --status CURRENT \
            --sort-by WWT --reverse --page-size 50 -J > report.json

      - name: Upload report
        uses: actions/upload-artifact@v4
        with:
          name: nsip-report
          path: report.json
```

---

## Step 6: Process Output with Standard Unix Tools

### CSV Conversion

Convert search results to CSV using `jq`:

```bash
nsip search --breed-id 486 --status CURRENT -J | \
    jq -r '.results[] | [.lpnId, .breed, .status, .bwt, .wwt, .ywt] | @csv'
```

### Filter and Count

```bash
# Count current males in a breed
nsip search --breed-id 486 --gender Male --status CURRENT -J | jq '.total_count'

# Find animals born in a specific year
nsip search --breed-id 486 --born-after 2022-01-01 --born-before 2022-12-31 -J | \
    jq '.total_count'
```

### Combine with Other Tools

```bash
# Compare two animals and extract only differing traits
diff <(nsip details 430735-0032 -J | jq '.traits') \
     <(nsip details 430735-0041 -J | jq '.traits')
```

---

## Step 7: Handle Errors Gracefully

### Retry on Transient Failures

```bash
fetch_with_retry() {
    local id="$1"
    local max_retries=3
    local attempt=0

    while [ "$attempt" -lt "$max_retries" ]; do
        if result=$(nsip details "$id" -J 2>/dev/null); then
            echo "$result"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep $((attempt * 2))
    done

    echo "Failed to fetch $id after $max_retries attempts" >&2
    return 1
}
```

Note: The `nsip` CLI has built-in retry logic (3 retries by default), so script-level retries are a second layer for additional resilience.

### Validate JSON Output

```bash
result=$(nsip details 430735-0032 -J)
if echo "$result" | jq empty 2>/dev/null; then
    echo "$result" | jq '.lpn_id'
else
    echo "Invalid JSON response" >&2
fi
```

---

## Verify the Integration

1. Run your script with a known LPN ID and check the output format.
2. Test error handling with an invalid LPN ID to confirm graceful failure.
3. In CI, check the exit code: `nsip` returns non-zero on errors.

---

## See Also

- [How to Export JSON](EXPORT-JSON.md) -- JSON output details
- [How to Batch Query Multiple Animals](BATCH-QUERY.md) -- batch processing patterns
- [How to Filter Search Results](FILTER-SEARCH-RESULTS.md) -- narrowing queries for scripts

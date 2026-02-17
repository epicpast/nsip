# Flock Action Workflow

> **Workflow:** `.github/workflows/flock-action.yml`  
> **Trigger:** Issues labeled `flock-action`  
> **Agent:** `copilot-swe-agent`  
> **Purpose:** Automated NSIP breeding analysis via GitHub Issues

## Overview

The Flock Action workflow enables users to request automated breeding analyses by filling out a structured GitHub issue form. When an issue with the `flock-action` label is opened, the workflow automatically assigns the Copilot coding agent, which:

1. Parses the issue body to extract parameters
2. Queries the NSIP database via MCP tools
3. Generates a formatted report with tables and recommendations
4. Creates a pull request with report artifacts

This workflow bridges the gap between non-technical users (sheep breeders) and the NSIP API, making advanced genetic analysis accessible via a simple web form.

---

## Workflow Architecture

### Trigger Conditions

```yaml
on:
  issues:
    types: [opened]
```

The workflow activates when:
- A new issue is created
- The issue has the `flock-action` label (auto-applied by the issue template)

### Permissions

```yaml
permissions:
  issues: write
```

Minimal permissions required only to assign the agent to the issue.

### Steps

1. **Check Label:** Conditional job only runs if `flock-action` label is present
2. **Assign Agent:** Uses GitHub API to assign `copilot-swe-agent` to the issue

The agent then operates independently with its own permissions and workflow logic.

---

## Issue Form Template

Location: `.github/ISSUE_TEMPLATE/flock-action.yml`

### Fields

| Field | Type | Required | Purpose |
|---|---|---|---|
| **Action** | Dropdown | Yes | Selects analysis type (6 options) |
| **LPN IDs** | Textarea | Yes | Animal identifiers (one per line) |
| **Trait Weights** | Textarea | No | Custom weights for Rank Animals (format: `TRAIT:weight`) |
| **Sort Trait** | Input | No | Trait abbreviation for Compare Animals sorting |
| **Notes** | Textarea | No | Additional context or breeding goals |

### Validation

- **Action:** Must select from predefined list
- **LPN IDs:** Required, free-form (agent validates against NSIP database)
- **Trait Weights:** Optional, validated by agent against trait abbreviation list
- **Sort Trait:** Optional, validated by agent against trait abbreviation list

---

## Agent Instructions

Location: `.github/instructions/flock-action.instructions.md`

The agent follows a structured workflow for each action type:

### 1. Parse Issue Body

Extract fields using regex or structured parsing:
- Action type from dropdown selection
- LPN IDs (split by newline, trim whitespace)
- Trait weights (parse `TRAIT:weight` format)
- Sort trait (single string)
- Notes (free-form text)

### 2. Execute Action Logic

Six distinct action types with specific tool call sequences:

#### Mating Recommendations
```
For each ewe LPN ID:
  1. Call `details` to get breed
  2. Call `mating_recommendations` with LPN and breed ID
  3. For each recommended sire, call `inbreeding_check`
```

#### Evaluate Flock
```
1. Call `details` for every LPN ID
2. Extract breed ID from first animal
3. Call `trait_ranges` for breed context
4. Calculate breed-relative percentiles
```

#### Compare Animals
```
1. Call `compare` with all LPN IDs
2. If sort_trait provided, sort results by that trait
3. Call `trait_ranges` for breed context
```

#### Rank Animals
```
1. Parse trait weights from issue
2. Call `details` for each LPN ID
3. Compute weighted score: sum(trait_value * weight * accuracy / 100)
4. Sort animals by score descending
5. Call `trait_ranges` for breed context
```

#### Inbreeding Matrix
```
1. For every unique pair (sire, dam):
   Call `inbreeding_check`
2. Build NxN matrix with COI values
3. Apply traffic-light ratings
```

#### Flock Profile
```
1. Call `details` for each LPN ID
2. Compute:
   - Total count
   - Gender distribution
   - Average EBVs per trait
3. Call `trait_ranges` for breed averages
4. Compare flock averages vs breed midpoints
```

### 3. Generate Artifacts

Create output in `reports/{YYYY-MM-DD}-{action-slug}/`:

**report.md:**
- Header with issue link (`Closes #N`)
- Executive summary
- Data tables (markdown format)
- Recommendations
- Trait glossary footnote

**data.csv:**
- Machine-readable export
- Column structure varies by action type
- See [CSV schemas](#csv-schemas) below

### 4. Create Pull Request

- **Branch:** `flock-action/{issue_number}-{action-slug}`
- **Title:** `[Flock Action] {Action Name} — {N} animals`
- **Body:** Summary + issue link + artifact list
- **Draft:** Yes (requires manual review before merge)

---

## MCP Tools Used

The agent relies on 13 NSIP MCP tools documented in `.github/instructions/nsip-mcp.instructions.md`:

| Tool | Purpose | Required Parameters |
|---|---|---|
| `search` | Find animals by criteria | `breed_id`, `status`, `gender`, etc. |
| `details` | Fetch full EBV profile | `animal_id` |
| `lineage` | Retrieve pedigree tree | `animal_id` |
| `progeny` | List offspring | `animal_id` |
| `profile` | Combined details + lineage + progeny | `animal_id` |
| `breed_groups` | List all breeds | None |
| `trait_ranges` | Get breed-wide min/max | `breed_id` |
| `compare` | Side-by-side comparison | `animal_ids` (2-5 items) |
| `rank` | Weighted composite ranking | `breed_id`, `weights` |
| `inbreeding_check` | Calculate COI | `sire_id`, `dam_id` |
| `mating_recommendations` | Find optimal mates | `animal_id`, `breed_id` |
| `flock_summary` | Aggregate flock stats | `flock_id` |
| `database_status` | Check last update | None |

---

## CSV Schemas

### Mating Recommendations

```csv
ewe_lpn,recommended_sire_lpn,rank_score,coi,predicted_bwt,predicted_wwt,predicted_pwwt
400123,500456,95.2,0.0312,8.5,45.2,72.1
400123,500789,93.7,0.0625,8.8,44.9,71.8
```

### Evaluate Flock

```csv
lpn_id,breed,gender,status,bwt,bwt_acc,wwt,wwt_acc,pwwt,pwwt_acc
400123,Dorper,Female,CURRENT,8.2,72,-1.5,68,3.2,65
400456,Dorper,Male,CURRENT,9.1,85,5.3,82,8.7,78
```

### Compare Animals

```csv
lpn_id,trait,value,accuracy
400123,BWT,8.2,72
400123,WWT,-1.5,68
400456,BWT,9.1,85
400456,WWT,5.3,82
```

### Rank Animals

```csv
lpn_id,total_score,bwt_contribution,wwt_contribution,pwwt_contribution
400456,87.3,-9.1,42.4,54.0
400123,72.1,-8.2,-12.0,92.3
```

### Inbreeding Matrix

```csv
sire_lpn,dam_lpn,coi,rating
500456,400123,0.0312,Green
500456,400456,0.0625,Yellow
500789,400123,0.1250,Red
```

### Flock Profile

```csv
metric,value
total_animals,42
male_count,8
female_count,34
avg_bwt,8.5
avg_wwt,2.3
avg_pwwt,5.7
```

---

## Agent Behavior

### Error Handling

The agent handles these error scenarios:

1. **Invalid LPN ID:** Comments on the issue with specific invalid IDs
2. **MCP tool timeout:** Retries with exponential backoff (max 3 attempts)
3. **Missing breed data:** Falls back to generic trait ranges
4. **Malformed trait weights:** Posts parsing errors to issue comments
5. **Insufficient animals for comparison:** Requires 2-5 animals for Compare action

### Formatting Rules

From `nsip-mcp.instructions.md`:

1. Always use markdown tables for EBV data (never raw JSON)
2. Show accuracy alongside every EBV: `+9.6 (68%)`
3. Use traffic-light indicators for COI (Green/Yellow/Red)
4. Include breed ranges for context
5. Round EBV values to 2 decimal places
6. Use trait natural units (lbs, mm, lambs, eggs/g)

### Report Structure

All reports follow this template:

````markdown
# {Action Name} Report

**Issue:** #{issue_number}  
**Date:** {YYYY-MM-DD}  
**Animals:** {N}  
**Breed:** {breed_name}

## Summary

{Executive summary paragraph}

## Results

{Data tables}

## Recommendations

{Actionable breeding advice}

## Glossary

- **BWT:** Birth Weight (lbs) — Select for moderate values
- **WWT:** Weaning Weight (lbs) — Select for higher values
- ...
````

---

## Security Considerations

### Input Validation

- LPN IDs are validated against NSIP database (no SQL injection risk)
- Trait weights are parsed with strict regex (no code injection)
- File paths are constructed with safe string concatenation

### Rate Limiting

- Agent respects NSIP API rate limits (built into `NsipClient`)
- MCP tool calls are sequential (no parallel flood)
- Timeout thresholds prevent runaway queries

### Authentication

- Agent uses repository-scoped `GITHUB_TOKEN` (no elevated privileges)
- NSIP API is public (no credentials required)
- MCP server runs in isolated subprocess

---

## Monitoring and Observability

### Success Metrics

Track these indicators via GitHub Actions logs:

- **Issues processed:** Count of `flock-action` issues opened
- **PRs created:** Count of successful report generations
- **Merge rate:** Percentage of draft PRs merged
- **Turnaround time:** Median time from issue open to PR creation

### Failure Modes

Monitor for:

- Agent assignment failures (GitHub API errors)
- MCP tool timeouts (database unresponsive)
- Parsing errors (malformed issue bodies)
- Insufficient data (missing breed, invalid LPN)

### Logs

Agent output is visible in:
1. Issue comments (progress updates)
2. PR body (summary of tool calls)
3. GitHub Actions workflow logs (detailed trace)

---

## Customization

### Adding New Action Types

1. Update `.github/ISSUE_TEMPLATE/flock-action.yml`:
   ```yaml
   - type: dropdown
     id: action
     attributes:
       options:
         - Your New Action
   ```

2. Add logic to `.github/instructions/flock-action.instructions.md`:
   ```markdown
   ### Your New Action
   1. Call `tool_name` with params
   2. Process results
   3. Generate output
   ```

3. Define CSV schema in this document

### Modifying Report Format

Edit the formatting rules in `.github/instructions/flock-action.instructions.md`:
- Change table layouts
- Add charts or visualizations
- Adjust trait glossary

### Tuning Agent Behavior

Adjust thresholds in agent instructions:
- COI traffic-light boundaries (currently 6.25%, 12.5%)
- Minimum accuracy for trait inclusion (currently no threshold)
- Number of recommendations per action (currently variable)

---

## Troubleshooting

### Agent Not Assigned

**Symptom:** Issue opened but agent never assigned.

**Diagnosis:**
1. Check if `flock-action` label is present
2. Verify workflow permissions in repository settings
3. Review Actions logs for API errors

**Solution:**
```bash
# Manually assign agent
gh api --method POST "/repos/{owner}/{repo}/issues/{number}/assignees" \
  -f "assignees[]=copilot-swe-agent"
```

### No PR Created

**Symptom:** Agent assigned but no PR appears after 30+ minutes.

**Diagnosis:**
1. Check issue comments for agent errors
2. Review agent workflow logs in Actions tab
3. Verify MCP server is accessible

**Solution:**
- Re-run agent by commenting: `@copilot-swe-agent please retry`
- Check `.mcp.json` configuration
- Verify binary is built: `cargo build --release`

### Incorrect Data

**Symptom:** Report contains wrong EBVs or recommendations.

**Diagnosis:**
1. Verify LPN IDs are correct
2. Cross-reference with NSIP web portal
3. Check database last-updated: `nsip date-updated`

**Solution:**
- Update LPN IDs in a new issue
- Comment on PR with specific discrepancies
- Open bug report if database sync issue

---

## Performance Considerations

### Scalability

- **Small flocks (< 10 animals):** ~5 minutes
- **Medium flocks (10-50 animals):** ~10 minutes
- **Large flocks (50-100 animals):** ~15 minutes
- **Very large flocks (100+ animals):** Consider batching into multiple issues

### Optimization

Reduce turnaround time by:
1. Pre-building MCP server binary in CI
2. Caching trait ranges per breed
3. Parallelizing independent MCP tool calls

### Rate Limits

NSIP API limits:
- **Requests per minute:** 60 (built-in throttling)
- **Concurrent connections:** 5 (enforced by client)
- **Max page size:** 100 animals per query

---

## Related Workflows

- **Issue Triage:** Auto-labels and assigns issues (`.github/workflows/issue-triage.lock.yml`)
- **CI Doctor:** Monitors workflow health (`.github/workflows/ci-doctor.lock.yml`)
- **Update Docs:** Keeps documentation in sync (`.github/workflows/update-docs.lock.yml`)

---

## Future Enhancements

Planned improvements:

1. **Interactive dashboards:** Web UI for visualizing reports
2. **Email notifications:** Send report PDFs to issue authors
3. **Multi-breed support:** Compare animals across breeds
4. **Genomic predictions:** Integrate SNP data when available
5. **Historical tracking:** Compare annual flock progress

---

## References

- [Flock Action User Guide](../how-to/FLOCK-ACTION.md) — End-user documentation
- [NSIP MCP Tools Reference](../reference/MCP-TOOLS.md) — Technical MCP tool specs
- [Agent Instructions](.github/instructions/flock-action.instructions.md) — Agent workflow logic
- [MCP Server Documentation](../docs/MCP.md) — Full MCP server API reference

---

**Maintainer Notes:**  
This workflow was introduced in commit `fd0db5f` to replace the gh-aw flock-action workflow, which failed due to GitHub MCP server connectivity issues. The Copilot coding agent approach has proven more reliable and provides native GitHub API access.

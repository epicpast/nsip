# How to Submit a Flock Action Request

> **Audience:** Sheep breeders, flock managers, and NSIP database users  
> **Time to complete:** 5 minutes  
> **Prerequisites:** GitHub account, LPN IDs of animals you want to analyze

## Overview

The Flock Action workflow lets you request automated breeding analyses by simply filling out a GitHub issue form. A specialized AI agent will process your request, query the NSIP database via MCP tools, and deliver a comprehensive report as a pull request.

## What You'll Need

- **GitHub account** with access to this repository
- **LPN IDs** of the animals you want to analyze (one or more)
- **Action type** — choose from six analysis options (see below)
- **Optional parameters** depending on the action type

---

## Available Actions

### 1. Mating Recommendations

**Purpose:** Find the best sire matches for your ewes based on genetic compatibility and breeding goals.

**What you provide:**
- LPN IDs of ewes (one per line)

**What you get:**
- Ranked list of recommended sires for each ewe
- Predicted offspring EBVs (Birth Weight, Weaning Weight, etc.)
- Inbreeding coefficient (COI) for each pairing with traffic-light ratings
- Breed-relative trait context

**Best for:** Planning mating season, selecting replacement sires

---

### 2. Evaluate Flock

**Purpose:** Get a comprehensive assessment of individual animals in your flock.

**What you provide:**
- LPN IDs of animals (any gender, any quantity)

**What you get:**
- Full EBV profiles for each animal
- Accuracy percentages for all traits
- Breed-relative rankings (e.g., "top 15% for WWT")
- Gender and status breakdown

**Best for:** Annual flock evaluation, identifying culling candidates, assessing replacement stock

---

### 3. Compare Animals

**Purpose:** Side-by-side trait comparison of 2-5 animals.

**What you provide:**
- LPN IDs (2-5 animals)
- Optional: Sort trait (e.g., `PWWT`)

**What you get:**
- Comparison table with all EBV traits
- Highlighted differences
- Breed context for each trait
- Optional sorting by a specific trait

**Best for:** Deciding between replacement sires, selecting show animals, evaluating progeny groups

---

### 4. Rank Animals

**Purpose:** Score and rank animals using a weighted composite index.

**What you provide:**
- LPN IDs of animals to rank
- Trait weights (format: `TRAIT:weight`, one per line)

**Example weights:**
````
WWT:0.3
PWWT:0.3
PFAT:-0.2
PEMD:0.2
````

**What you get:**
- Animals ranked by weighted composite score
- Individual trait contributions to total score
- Breed-relative context
- CSV export for further analysis

**Best for:** Custom selection indexes, terminal sire selection, balanced breeding programs

---

### 5. Inbreeding Matrix

**Purpose:** Calculate inbreeding coefficients (COI) for all possible pairings within a group.

**What you provide:**
- LPN IDs of candidate animals (typically males and females)

**What you get:**
- Matrix of COI values for every unique pairing
- Traffic-light ratings (Green < 6.25%, Yellow 6.25-12.5%, Red > 12.5%)
- Safe and risky mating recommendations

**Best for:** Closed flock management, avoiding inbreeding depression, maintaining genetic diversity

---

### 6. Flock Profile

**Purpose:** Statistical summary of an entire flock.

**What you provide:**
- LPN IDs of all animals in the flock

**What you get:**
- Total animal count
- Gender distribution
- Average EBVs across all traits
- Comparison against breed-wide averages
- Flock strengths and weaknesses

**Best for:** Annual reporting, flock benchmarking, identifying breeding priorities

---

## Step-by-Step Instructions

### 1. Navigate to Issues

Go to the repository's **Issues** tab:

```
https://github.com/zircote/nsip/issues
```

### 2. Create a New Issue

Click **"New issue"** and select the **"Flock Action"** template.

### 3. Fill Out the Form

#### Action (required)
Select one of the six action types from the dropdown.

#### LPN IDs (required)
Enter one LPN identifier per line. Example:

```
400123
400456
400789
```

#### Trait Weights (Rank Animals only)
If you selected "Rank Animals", provide trait weights in this format:

```
WWT:0.3
PWWT:0.3
PFAT:-0.2
PEMD:0.2
```

**Weight guidelines:**
- Positive weights favor higher values (e.g., `WWT:0.5` = select for heavier weaning weight)
- Negative weights favor lower values (e.g., `BWT:-0.2` = select against heavy birth weight)
- Weights don't need to sum to 1.0 (the agent will normalize them)

#### Sort Trait (Compare Animals only)
If you selected "Compare Animals" and want results sorted by a specific trait, enter the trait abbreviation (e.g., `PWWT`).

#### Notes (optional)
Provide any additional context for the analysis:
- Breeding goals (e.g., "Focus on growth traits")
- Environmental factors (e.g., "Hill country flock, prioritize hardiness")
- Special considerations

### 4. Submit the Issue

Click **"Submit new issue"**. The `flock-action` label will be automatically applied.

### 5. Wait for the Agent

The **Copilot coding agent** will be automatically assigned and will:
1. Parse your issue body
2. Query the NSIP database via MCP tools
3. Generate a formatted report with tables, charts, and recommendations
4. Create a pull request with artifacts in `reports/{YYYY-MM-DD}-{action-slug}/`

**Expected turnaround:** 5-15 minutes depending on the number of animals

### 6. Review the Pull Request

The agent will open a draft PR with:
- **Title:** `[Flock Action] {Action Name} — {N} animals`
- **Body:** Summary of findings and link back to your issue
- **Files:**
  - `reports/{date}-{action}/report.md` — Human-readable analysis
  - `reports/{date}-{action}/data.csv` — Machine-readable export

Review the report and CSV data. If you need adjustments, comment on the PR with your requested changes.

### 7. Merge the PR

Once satisfied, merge the pull request to add the report to the repository's permanent record.

---

## Trait Abbreviations Reference

| Abbreviation | Full Name | Unit | Selection Direction |
|---|---|---|---|
| **BWT** | Birth Weight | lbs | Lower (avoid dystocia) |
| **WWT** | Weaning Weight | lbs | Higher |
| **PWWT** | Post-Weaning Weight | lbs | Higher |
| **YWT** | Yearling Weight | lbs | Higher |
| **PFAT** | Post-Weaning Fat Depth | mm | Moderate |
| **PEMD** | Post-Weaning Eye Muscle Depth | mm | Higher |
| **NLB** | Number of Lambs Born | lambs | Higher (with caution) |
| **NWT** | Number of Lambs Weaned | lambs | Higher |
| **PWT** | Pounds Weaned | lbs | Higher |
| **DAG** | Dag Score | score | Lower (less manure staining) |
| **WEC** | Worm Egg Count | eggs/g | Lower (parasite resistance) |
| **FEC** | Faecal Egg Count | eggs/g | Lower (parasite resistance) |

See [Breed Groups and Traits](../explanation/BREED-GROUPS-AND-TRAITS.md) for detailed explanations.

---

## Troubleshooting

### "Invalid LPN ID" Error

**Symptom:** Agent reports that one or more LPN IDs are not found in the database.

**Solution:**
1. Verify the LPN ID is correct (no typos, correct format)
2. Check that the animal is registered in the NSIP database
3. Try searching via the CLI: `nsip search --animal-id {LPN}`

### "Missing Trait Weights" Error

**Symptom:** Rank Animals action fails with a parsing error.

**Solution:**
1. Ensure weights follow the `TRAIT:weight` format exactly
2. One trait-weight pair per line
3. Use valid trait abbreviations (see table above)
4. Weights can be positive or negative decimals

### Agent Timeout

**Symptom:** No PR appears after 30 minutes.

**Solution:**
1. Check the issue for agent comments or error messages
2. Verify the `flock-action` label is applied
3. Re-assign the agent by commenting: `@copilot-swe-agent please retry`

### Report Data Looks Wrong

**Symptom:** EBV values or recommendations don't match expectations.

**Solution:**
1. Verify the LPN IDs are correct
2. Check the database last-updated date: `nsip date-updated`
3. Cross-reference with the NSIP web portal
4. Comment on the PR with specific discrepancies

---

## Example Workflow

**Scenario:** You want to find the best sire for three of your top ewes.

1. Navigate to **Issues** → **New issue** → **Flock Action**
2. Select **"Mating Recommendations"** from the Action dropdown
3. Enter LPN IDs in the "LPN IDs" field:
   ```
   400123
   400456
   400789
   ```
4. Add optional notes:
   ```
   Focus on growth traits (WWT, PWWT). Avoid high birth weight.
   ```
5. Click **Submit new issue**
6. Wait ~10 minutes
7. Review the generated PR with:
   - Ranked sire recommendations for each ewe
   - Predicted offspring traits
   - COI traffic-light indicators
8. Merge the PR to save the report

---

## Advanced Usage

### Combining Multiple Actions

You can submit multiple Flock Action issues for the same set of animals:
1. **Evaluate Flock** to get baseline data
2. **Rank Animals** with custom weights
3. **Inbreeding Matrix** to check genetic diversity
4. **Mating Recommendations** for top-ranked females

Each action creates a separate PR, allowing you to build a comprehensive breeding plan.

### CSV Export for Spreadsheets

Every report includes a `data.csv` file. Import this into Excel, Google Sheets, or LibreOffice for:
- Custom charts and visualizations
- Pivot tables for deeper analysis
- Integration with herd management software

### Automating Seasonal Analyses

Create a saved template for your annual breeding season:
1. Fork this repository (or create issues in your own fork)
2. Maintain a CSV file with your flock's LPN IDs
3. Copy/paste LPN IDs into Flock Action issues each season
4. Archive reports by year in separate folders

---

## Related Documentation

- [NSIP MCP Tools Reference](../reference/MCP-TOOLS.md) — Technical details of the underlying MCP server
- [Understanding EBVs](../explanation/EBV-EXPLAINED.md) — Learn how to interpret Estimated Breeding Values
- [Compare Animals Guide](COMPARE-ANIMALS.md) — Manual comparison via CLI or library
- [Use MCP Tools](USE-MCP-TOOLS.md) — Direct MCP tool invocation from AI assistants

---

## Need Help?

- **Documentation issues:** Open a documentation bug report
- **Feature requests:** Suggest new Flock Action types or report formats
- **Technical support:** Check the [Troubleshooting](#troubleshooting) section or open a support issue

---

**Next Steps:**  
Ready to try it out? Head to [Issues → New issue → Flock Action](../../issues/new?template=flock-action.yml) and submit your first request!

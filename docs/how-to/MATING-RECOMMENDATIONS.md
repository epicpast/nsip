---
diataxis_type: how-to
---

# How to Make Data-Driven Mating Decisions

> **Problem:** You need to select the best mates for your breeding stock by comparing EBV traits, ranking candidates, checking inbreeding risk, and generating mating recommendations.

**Prerequisites:**

- `nsip` CLI installed, or the NSIP MCP server configured in your AI assistant
- LPN IDs for the animals you want to evaluate
- Breed ID for your breed (use `nsip breeds` or the `breed_groups` MCP tool to look it up)

---

## Compare animals side-by-side

Use side-by-side comparison to see how candidate animals differ on the traits that matter to your breeding program.

### CLI

Compare two to five animals by their LPN IDs:

```bash
nsip compare 430735-0032 430735-0041 --traits BWT,WWT,YWT
```

Add more IDs to compare up to five animals at once:

```bash
nsip compare 430735-0032 430735-0041 430735-0058 --traits BWT,WWT,YWT,EMD,NLB
```

Omit `--traits` to see all 13 EBV traits. Add `-J` for JSON output.

### MCP

Call the `compare` tool with an array of LPN IDs:

```json
{
  "tool": "compare",
  "arguments": {
    "lpn_ids": ["430735-0032", "430735-0041"],
    "traits": "BWT,WWT,YWT"
  }
}
```

The response includes each animal's EBV values aligned by trait, so you can spot differences at a glance.

**Tip:** Start with a broad comparison (no trait filter), then narrow to the traits relevant to your breeding objective once you know which animals are competitive.

---

## Rank animals by weighted traits

Ranking lets you find the top-N animals in a breed according to your own trait priorities. You assign weights to traits: positive weights favor higher values, negative weights favor lower values.

### MCP

Call the `rank` tool with your breed ID and a weights object:

```json
{
  "tool": "rank",
  "arguments": {
    "breed_id": 486,
    "weights": {
      "BWT": -1.0,
      "WWT": 2.0,
      "YWT": 1.5,
      "EMD": 1.0
    },
    "gender": "Male",
    "status": "CURRENT",
    "top_n": 5
  }
}
```

This example finds the top 5 current rams for a terminal sire objective: penalizing high birth weight while prioritizing weaning weight, yearling weight, and eye muscle depth.

**Common weight profiles:**

| Objective | Suggested weights |
|-----------|-------------------|
| Terminal sire | `BWT: -1.0, WWT: 2.0, YWT: 1.5, EMD: 1.0` |
| Maternal sire | `NLB: 2.0, NWT: 2.0, PWT: 1.5, BWT: -0.5` |
| Dual purpose | `WWT: 1.5, YWT: 1.0, NLB: 1.0, NWT: 1.0, BWT: -0.5` |
| Parasite resistance | `WEC: -2.0, FEC: -2.0, WWT: 1.0` |

Each animal's composite score is calculated as the sum of `(trait_value * weight * accuracy / 100)` across all weighted traits, so higher-accuracy animals naturally rank higher.

---

## Check inbreeding risk

Before committing to a mating, check whether the sire and dam share recent ancestors. The `inbreeding_check` tool computes Wright's coefficient of inbreeding (COI) by walking both pedigrees and finding common ancestors.

### MCP

Call the `inbreeding_check` tool with sire and dam LPN IDs:

```json
{
  "tool": "inbreeding_check",
  "arguments": {
    "sire_id": "430735-0032",
    "dam_id": "430735-0089"
  }
}
```

The response includes three pieces of information:

1. **coefficient** -- the COI as a decimal (e.g., `0.03125` means 3.125%)
2. **rating** -- a traffic-light classification:

   | Rating | COI range | Meaning |
   |--------|-----------|---------|
   | Green | < 6.25% | Acceptable -- proceed |
   | Yellow | 6.25% -- 12.5% | Elevated -- consider alternatives |
   | Red | > 12.5% | High -- generally avoid |

3. **shared_ancestors** -- a list of common ancestors with path depths from each parent

**Example response:**

```json
{
  "coefficient": 0.03125,
  "rating": "Green",
  "shared_ancestors": [
    {
      "lpn_id": "410220-0015",
      "sire_depth": 2,
      "dam_depth": 2
    }
  ]
}
```

In this example, the sire and dam share one grandparent (depth 2 on both sides), producing a COI of 3.125% -- within the acceptable range.

**Tip:** Run inbreeding checks on your top candidates from the ranking step before making a final decision. A high-scoring animal is not worth using if it pushes inbreeding above your threshold.

---

## Get mating recommendations

The `mating_recommendations` tool combines everything above into a single step: it searches the breed for potential mates, checks inbreeding for each candidate, and ranks them by trait complementarity.

### MCP

Call the `mating_recommendations` tool with the animal you want to find mates for:

```json
{
  "tool": "mating_recommendations",
  "arguments": {
    "animal_id": "430735-0032",
    "breed_id": 486,
    "target_traits": "WWT,EMD,NLB",
    "max_results": 3
  }
}
```

**Parameters:**

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| `animal_id` | yes | -- | LPN ID of the animal to find mates for |
| `breed_id` | yes | -- | Breed to search within |
| `target_traits` | no | `WWT,BWT,NLB` | Traits to optimize (comma-separated) |
| `max_results` | no | 5 | Number of recommendations to return |

When you omit `target_traits`, the tool defaults to weaning weight (positive), birth weight (negative), and number of lambs born (positive). Traits where lower is preferred -- BWT, DAG, WEC, FEC -- automatically receive negative weights.

**Example response:**

```json
[
  {
    "mate_lpn_id": "430735-0089",
    "rank_score": 18.42,
    "coi": {
      "coefficient": 0.015,
      "rating": "Green"
    },
    "predicted_offspring_ebvs": {
      "BWT": 0.15,
      "WWT": 11.3,
      "EMD": 1.8,
      "NLB": 0.12
    }
  }
]
```

Each recommendation includes:

- **rank_score** -- composite score reflecting trait complementarity
- **coi** -- inbreeding check result (candidates with Red ratings are excluded)
- **predicted_offspring_ebvs** -- midparent EBV estimates for the traits you specified

---

## Putting it all together

A typical breeding decision workflow:

1. **Rank** your breed to identify the top candidates for your objective.
2. **Compare** your shortlisted animals side-by-side, filtering to the traits you care about.
3. **Check inbreeding** for each candidate pairing you are considering.
4. **Get recommendations** to let the tool automate steps 1--3 and surface the best mates.

You can use the MCP guided prompts for a more conversational workflow. Ask your AI assistant to use the `evaluate-ram`, `evaluate-ewe`, or `plan-mating` prompts for structured breeding assessments that incorporate all of these tools automatically.

---

## Further reading

- [MCP Tools Reference](../reference/MCP-TOOLS.md) -- full parameter reference for all 13 tools
- [From Data to Decisions](../explanation/DATA-TO-DECISIONS.md) -- background on how EBVs translate to breeding decisions
- [Compare Animals](COMPARE-ANIMALS.md) -- detailed guide focused on the compare workflow, including library usage

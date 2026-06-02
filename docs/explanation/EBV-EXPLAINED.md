---
diataxis_type: explanation
---
# Understanding Estimated Breeding Values (EBVs)

> EBVs are the foundation of genetic improvement in sheep. This guide explains what they are, how they work, and why they matter for breeding decisions.

---

## What is an EBV?

An **Estimated Breeding Value** (EBV) predicts the genetic merit an animal will pass to its offspring for a specific trait. It is not a measurement of the animal's own performance -- it is an estimate of the *average genetic contribution* to its progeny.

EBVs are expressed as deviations from a breed average baseline (typically zero). A positive EBV indicates above-average genetic potential; a negative EBV indicates below-average. The units match the trait being measured.

For example, if a ram has a Weaning Weight (WWT) EBV of +2.5 lbs, his offspring are expected to be 2.5 lbs heavier at weaning than those of an average ram in the same breed, all else being equal. Because each parent contributes half the genetics, the expected offspring advantage is actually half the difference between the two parents' EBVs. This half-EBV concept is sometimes called an **Expected Progeny Difference (EPD)** -- the EPD is simply half the EBV difference between two animals.

---

## Why EBVs, Not Raw Performance?

Raw performance (phenotype) reflects both genetics and environment. A lamb raised in excellent conditions may outperform a genetically superior lamb raised in poor conditions. EBVs strip away environmental effects to isolate the heritable genetic component.

This matters because:

- **Genetics are heritable; environment is not.** A well-fed lamb does not pass its nutrition to offspring.
- **Fair comparison requires adjustment.** Animals from different flocks, years, and management systems can be compared on a level playing field.
- **Selection response is predictable.** Selecting on EBVs produces consistent genetic improvement across generations.

---

## NSIP EBV Traits

NSIP evaluates a range of traits organized into functional categories. Each trait has a standard abbreviation used throughout the API and CLI. Not all traits are evaluated for every breed -- trait availability depends on the breed group and data collection practices.

> **Note on units:** The NSIP Search API reports growth traits in **lbs** (US customary) while the underlying LAMBPLAN evaluation system in Australia uses kg. The `nsip` CLI and library return values in the units provided by the API (lbs for growth, mm for carcass). The tables below show the API units.

### Growth Traits

| Abbreviation | Trait | Unit | Selection Direction |
|---|---|---|---|
| BWT | Birth Weight | lbs | Lower preferred (reduces dystocia) |
| WWT | Weaning Weight (60 days) | lbs | Higher preferred |
| MWWT | Maternal Weaning Weight | lbs | Higher preferred (more milk) |
| PWWT | Post-Weaning Weight | lbs | Higher preferred |
| YWT | Yearling Weight | lbs | Higher preferred |

Birth weight is unusual: lower EBVs are generally preferred because heavier birth weights increase the risk of lambing difficulty (dystocia). The other growth traits follow the typical "higher is better" pattern for meat production. **MWWT (Maternal Weaning Weight)** is a distinct trait from WWT -- it measures the dam's genetic contribution to lamb weaning weight through milk production and maternal care, rather than the lamb's own direct growth potential.

### Carcass Traits

Carcass traits are standardized to a reference body weight of 55 kg (121 lbs) to allow fair comparison across animals measured at different weights.

| Abbreviation | Trait | Unit | Selection Direction |
|---|---|---|---|
| FAT (CF) | Post-Weaning Fat Depth | mm | Lower preferred (less fat) |
| EMD | Post-Weaning Eye Muscle Depth | mm | Higher preferred |

Eye muscle depth measures the loin muscle cross-section and correlates with lean meat yield. Fat depth is measured as subcutaneous fat thickness via ultrasound -- lower values indicate leaner carcasses. NSIP also provides a **Carcass Plus** composite that combines EMD, FAT, and PWWT into a single carcass merit value.

### Reproduction Traits

| Abbreviation | Trait | Unit | Selection Direction |
|---|---|---|---|
| NLB | Number of Lambs Born | % above breed avg | Higher (with caution) |
| NWT | Number of Lambs Weaned | % above breed avg | Higher preferred |
| SC | Scrotal Circumference | mm | Higher preferred (fertility indicator) |

NLB drives prolificacy but must be balanced against lamb survival -- triplets and quads have higher mortality. NWT captures the combined effect of prolificacy and lamb survival, making it a more practical selection target than NLB alone. SC (Scrotal Circumference) is a male fertility indicator -- higher values correlate with improved fertility in both the ram and his daughters.

### Parasite Resistance Traits

| Abbreviation | Trait | Unit | Selection Direction |
|---|---|---|---|
| WEC | Weaning Fecal Egg Count | % | Lower/negative preferred |
| FEC | Post-Weaning Fecal Egg Count | % | Lower/negative preferred |

WEC and FEC measure parasite resistance as a percentage relative to the breed average. **Negative values indicate greater resistance.** For example, a ram with a WEC of -90% has the potential to reduce worm burden in his lambs by approximately 45% (since half the genetics pass to offspring). Selecting for parasite resistance reduces the need for anthelmintic (deworming) treatments, slows the development of drug-resistant parasite populations, and improves animal welfare.

### Wool Traits (Wool Breeds Only)

For wool-producing breeds, additional traits may be evaluated including GFW (Greasy Fleece Weight), CFW (Clean Fleece Weight), FD (Fiber Diameter), SL (Staple Length), SS (Staple Strength), FDCV (Fiber Diameter CV), and CURV (Fiber Curvature). These traits are not relevant for hair sheep breeds.

---

## How EBVs Are Calculated

NSIP uses **BLUP (Best Linear Unbiased Prediction)**, the same statistical method used in cattle, pig, and poultry genetic evaluation worldwide.

### Data Inputs

BLUP combines three sources of information:

1. **Performance records** -- measured traits from the animal and its contemporaries (birth weights, weaning weights, ultrasound scans, etc.)
2. **Pedigree data** -- ancestry relationships linking animals to their sire, dam, and extended family. This allows information to flow between relatives.
3. **Genomic data** (when available) -- DNA marker information that refines the estimated genetic relationships between animals.

### What BLUP Does

The BLUP model simultaneously estimates:

- **Fixed effects** -- systematic environmental factors such as year of birth, flock of origin, age of dam, birth type (single/twin/triplet), and sex. These are "corrected out" so they do not bias the genetic estimates.
- **Random genetic effects** -- the actual breeding values. BLUP uses the pedigree (and genomic) relationships to borrow information from relatives, which is why a young animal with no progeny can still have an EBV based on its parents' and siblings' data.

The key property of BLUP is that it produces **unbiased** estimates -- the EBVs are not systematically too high or too low for any group of animals. This is what makes across-flock comparison valid.

### Genetic Connectedness

For EBVs to be comparable across flocks, flocks must share genetic links. This happens when:

- Rams are used across multiple flocks
- AI (artificial insemination) sires create connections
- Reference sires are shared through cooperative programs

Without connectedness, EBVs from different flocks cannot be meaningfully compared, even within the same breed evaluation.

---

## Understanding Accuracy

Every EBV in the NSIP system has an associated **accuracy** value. In the `nsip` crate, accuracy is stored as an integer percentage (0--100) in the `Trait` struct:

```rust
// From crates/models.rs
pub struct Trait {
    pub name: String,
    pub value: f64,
    pub accuracy: Option<i32>,  // Integer percentage 0-100
    pub units: Option<String>,
}
```

### What Accuracy Means

Accuracy measures the reliability of an EBV estimate -- how likely the EBV is to change as more data becomes available. It reflects the amount and quality of information behind the estimate.

| Accuracy Range | Interpretation | Typical Source |
|---|---|---|
| 0--29% | Low | Parent average only, no own or progeny data |
| 30--59% | Moderate | Own performance and/or genomic data |
| 60--79% | High | Some progeny records |
| 80--100% | Very high | Extensive progeny data across multiple flocks |

### Practical Implications

- **High-accuracy EBVs are stable.** An animal with 85% accuracy for WWT is unlikely to see its EBV change significantly with additional data.
- **Low-accuracy EBVs carry risk.** A young ram with 25% accuracy might look exceptional, but his true breeding value could be substantially different. Use low-accuracy EBVs for screening, not final selection.
- **Proven sires have high accuracy.** Rams with many progeny across multiple flocks accumulate accuracy quickly.
- **Accuracy increases over time.** As an animal accumulates its own records and progeny data, its accuracy rises.

### Where Accuracy Appears in the Data

Every trait the API returns carries its accuracy alongside its value, so accuracy is never something you have to compute or infer — it is reported per trait. In the `nsip` library, an animal fetched via `animal_details` exposes each trait through its `traits` map, and a trait's `accuracy` is an optional integer percentage; a missing accuracy simply means the API did not supply one for that trait. The practical reading is the one described above: treat a low-accuracy figure as a screening signal rather than a settled fact. For the commands and code that retrieve an animal's details and inspect trait accuracies, see [How to Compare Animals](../how-to/COMPARE-ANIMALS.md).

---

## Genetic Trend and Base Changes

EBV values are relative to a breed base that may shift over time as the breed average improves through selection. This has important consequences:

- **An EBV of +5 today is not the same as +5 ten years ago.** If the breed has improved, the base has shifted upward, and today's +5 represents a higher absolute genetic level.
- **Always compare animals from the same evaluation run.** The NSIP database is periodically re-evaluated, and all EBVs are recalculated together.
- **Check the database date** to confirm you are working with current evaluations. The NSIP API exposes the last-updated date of the evaluation run, and the `nsip` tooling surfaces it so you can confirm which run your numbers come from before drawing conclusions from them.

---

## Selection Indexes

Selecting on individual traits one at a time is inefficient and can cause problems. Pushing hard on weaning weight alone might inadvertently increase birth weight (and dystocia risk) because the traits are genetically correlated.

**Selection indexes** solve this by combining multiple EBVs into a single score, weighted by their economic importance and adjusted for genetic correlations between traits.

### How Indexes Work

An index assigns economic weights to each trait based on its impact on profitability. For example, a simplified terminal sire index might look like:

```
Index = (w1 x WWT) + (w2 x PWWT) + (w3 x EMD) - (w4 x FAT) - (w5 x BWT)
```

The weights (w1 through w5) are derived from economic modeling and genetic parameters (heritabilities and correlations). Negative weight on BWT means the index penalizes animals that increase birth weight.

### NSIP Indexes

NSIP provides several selection indexes tailored to different production systems:

- **USA MAT-HAIR Index** -- designed to maximize the total weight of lamb weaned per ewe lambing. It combines DWWT (Direct Weaning Weight), MWWT (Maternal Weaning Weight), NLB, and NWT, with NWT receiving the heaviest economic weighting. This index is used for hair sheep breeds such as Katahdin and Dorper.
- **USA Terminal Index** -- emphasizes growth and carcass traits for terminal sire breeds (Suffolk, Hampshire, Texel, etc.), prioritizing lean meat production.

The NSIP API provides index values through the lineage endpoint. The `LineageAnimal` struct includes:

- `us_index` -- the US index value (e.g., USA MAT-HAIR for hair breeds)
- `src_index` -- the SRC$ Index

These pre-calculated indexes save producers from having to compute their own weighted combinations.

### When to Use Indexes vs. Individual Traits

| Scenario | Approach |
|---|---|
| General flock improvement | Use the published index |
| Corrective mating (fixing a specific weakness) | Emphasize the relevant individual trait |
| Research or custom breeding objectives | Build a custom index with appropriate weights |

---

## Comparing Animals

EBV comparisons are only meaningful under specific conditions:

1. **Same breed.** EBVs are calculated within breed. A Katahdin with WWT +3.0 cannot be compared to a Suffolk with WWT +3.0 -- the breed bases are different.
2. **Same evaluation run.** EBVs from different evaluation dates may use different base adjustments.
3. **Consider accuracy.** When two animals have similar EBVs but different accuracies, the higher-accuracy animal is the safer choice.

These conditions are what make comparison meaningful rather than misleading: hold breed, evaluation run, and accuracy in mind, and the differences between animals reflect genetics rather than artifacts of the data. For the dedicated comparison command and the library calls that fetch multiple animals for side-by-side analysis, see [How to Compare Animals](../how-to/COMPARE-ANIMALS.md).

---

## Common Misconceptions

**"A higher EBV is always better."**
Not true. For BWT, lower is preferred. For FAT, lower is preferred (less fat). For WEC and FEC, lower/negative values are preferred (indicating greater parasite resistance). Always check the selection direction for each trait.

**"EBVs predict an animal's own performance."**
EBVs predict genetic contribution to offspring, not the animal's own phenotype. A ewe with a high WWT EBV may have been a light lamb herself if she was raised in poor conditions.

**"I can compare EBVs across breeds."**
EBVs are breed-specific. The genetic base, heritability estimates, and evaluation models differ between breeds. Cross-breed comparisons require special across-breed evaluation methodologies that NSIP does not currently provide.

**"Low-accuracy EBVs are useless."**
They are less reliable, but still the best available estimate of an animal's genetic merit. A low-accuracy EBV from BLUP is better than no genetic information at all. Use them as screening tools while waiting for more data.

**"Once calculated, an EBV never changes."**
EBVs are re-estimated with each evaluation run as new data arrives. An animal's EBV can change -- sometimes substantially for young animals with low accuracy -- as progeny records accumulate.

---

## Further Reading

- [Genetic Evaluation](GENETIC-EVALUATION.md) -- deeper dive into BLUP methodology and connectedness
- [Breed Groups and Traits](BREED-GROUPS-AND-TRAITS.md) -- understanding how breeds and traits are organized
- [NSIP Data Model](NSIP-DATA-MODEL.md) -- how the API structures animal, lineage, and progeny data
- [Data to Decisions](DATA-TO-DECISIONS.md) -- connecting API data to real-world breeding decisions
- [How to Compare Animals](../how-to/COMPARE-ANIMALS.md) -- step-by-step comparison guide
- [Getting Started Tutorial](../tutorials/GETTING-STARTED.md) -- hands-on introduction to the NSIP API
- [Error Handling Reference](../reference/ERROR-HANDLING.md) -- complete error type reference

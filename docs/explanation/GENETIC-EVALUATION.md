---
diataxis_type: explanation
---
# Genetic Evaluation

> How the NSIP system transforms raw performance data into Estimated Breeding Values using BLUP, and why the methodology matters for breeding decisions.

---

## What is Genetic Evaluation?

Genetic evaluation is the statistical process of estimating an animal's genetic merit from available data. The NSIP uses this process to produce Estimated Breeding Values (EBVs) for multiple traits across 23 sheep breeds in the United States.

NSIP was founded in 1986 and completed its Phase I development in 1990. **Across-flock evaluations** -- the ability to compare animals from different flocks on a common genetic scale -- began around 1993, a major milestone that enabled meaningful national genetic improvement. Today, NSIP operates through a cooperative agreement with **Sheep Genetics (Australia)**, where flock data is processed through **Pedigree Master** software and sent to **LAMBPLAN** for BLUP genetic analysis. Data is submitted every two weeks for ongoing EBV development.

The goal is to separate the heritable genetic component of a trait from the environmental influences. A lamb's weaning weight, for example, depends on its genetics, its dam's milk production, the pasture quality, the weather, and the management system. Genetic evaluation isolates the genetic signal from all that noise.

---

## BLUP: The Engine Behind EBVs

NSIP uses **Best Linear Unbiased Prediction** (BLUP), developed by Charles Henderson in the 1950s and now the standard method for livestock genetic evaluation worldwide.

### Why "Best Linear Unbiased Prediction"?

Each word in the name describes a specific statistical property:

- **Best** -- among all linear unbiased predictors, BLUP minimizes the prediction error variance. In practical terms, BLUP produces the most accurate estimates possible given the available data.
- **Linear** -- the estimated breeding values are linear functions of the observed data. This makes computation tractable even for very large datasets.
- **Unbiased** -- the estimates have no systematic bias. Animals from different flocks, years, or management systems are evaluated on a fair basis.
- **Prediction** -- breeding values are predicted (not directly observed), because we can never measure an animal's true genetic merit directly.

### The Mixed Model Equations

BLUP works by solving a system called the **mixed model equations**. In simplified form:

```
y = Xb + Zu + e
```

Where:

- **y** = vector of observed phenotypes (measured traits like birth weight, weaning weight)
- **X** = design matrix linking observations to fixed effects
- **b** = fixed effects (year, flock, sex, age of dam, birth type, etc.)
- **Z** = design matrix linking observations to animals
- **u** = random genetic effects (the breeding values we want to estimate)
- **e** = residual error (unexplained variation)

The system simultaneously solves for the fixed effects (b) and the breeding values (u). This simultaneous estimation is critical -- it prevents environmental effects from biasing the genetic estimates and prevents genetic differences from biasing the environmental corrections.

### The Role of the Relationship Matrix

BLUP uses a **genetic relationship matrix** (A) that encodes the pedigree relationships between all animals in the evaluation. This matrix allows information to flow between relatives:

- A sire's EBV is informed by his offspring's performance
- A young animal's EBV is informed by its parents' and siblings' data
- Even animals with no own performance data receive an EBV (parent average)

The relationship matrix is why pedigree accuracy matters. Incorrect parentage assignments (wrong sire or dam) introduce errors that propagate through the evaluation.

### Genomic Enhancement

When DNA marker data is available, BLUP can be extended to **GBLUP** (Genomic BLUP) or **single-step genomic evaluation** by replacing or supplementing the pedigree-based relationship matrix with a genomic relationship matrix. Single-step methods combine pedigree, phenotypic, and genomic information simultaneously, which is the direction modern genetic evaluation is moving. This improves accuracy because:

- Actual genetic sharing between relatives varies around the expected value (full siblings share 50% on average but the actual value ranges from roughly 38% to 62%)
- Genomic data captures the actual sharing, not just the expected value
- Animals without pedigree connections can be related through shared genomic segments

The `genotyped` field in `AnimalDetails` indicates whether an animal has genomic data incorporated into its evaluation.

---

## Fixed Effects: What Gets Corrected

BLUP removes the influence of known environmental factors so that the remaining variation reflects genetics. The major fixed effects in sheep evaluation include:

### Flock-Year-Season (Contemporary Group)

The most important fixed effect. Animals are compared only against contemporaries raised in the same flock, during the same time period, under the same management. A lamb from a high-performing flock is not credited for its superior environment.

### Age of Dam

Older ewes (2nd through 5th parity) typically produce heavier lambs than first-parity ewes, because they are physically more mature and produce more milk. BLUP adjusts for this so that a lamb from a maiden ewe is not penalized.

### Birth Type and Rearing Type

Twins and triplets are lighter at birth and weaning than singles, because they share uterine space and maternal resources. BLUP adjusts for birth type (single, twin, triplet) and rearing type (how many lambs the ewe actually raised).

### Sex

Males are typically heavier than females at all ages. Separate sex effects prevent this from biasing EBVs.

---

## Across-Flock Evaluation and Connectedness

A key feature of NSIP's BLUP evaluation is that EBVs are comparable across flocks within the same breed. This is possible only when flocks are **genetically connected** -- they share genetic material through common ancestors.

### How Connectedness Works

```
     Flock A          Flock B
       |                 |
       v                 v
    Ram X (AI sire) used in both flocks
       |                 |
       v                 v
    Progeny A        Progeny B
```

When Ram X produces offspring in both Flock A and Flock B, the evaluation can separate the effect of the ram's genetics from the effect of each flock's environment. Without this link, the evaluation cannot tell whether Flock A's lambs are heavier because of better genetics or better feed.

### Sources of Connectedness

- **AI (artificial insemination) sires** -- the most common source of cross-flock links
- **Reference sire programs** -- cooperative programs where multiple flocks use designated rams
- **Ram purchases** -- when a flock buys a ram from another flock and both record progeny
- **Genomic links** -- DNA data can reveal relationships between animals in different flocks

### Why Disconnected Comparisons Fail

If two flocks have no genetic connections, their EBVs exist on independent scales. Flock A's "best" ram (EBV +5) might actually be genetically inferior to Flock B's "worst" ram (EBV -2) if Flock B's genetics are substantially better overall. The NSIP evaluation flags the degree of connectedness to help producers interpret cross-flock comparisons.

---

## Multi-Trait Evaluation

NSIP evaluates multiple traits simultaneously rather than one at a time. This multi-trait approach is important because:

### Genetic Correlations

Traits are not independent. Selecting for higher weaning weight also tends to increase birth weight, because the genes affecting growth are partially shared. The major genetic correlations in sheep include:

| Trait Pair | Correlation Direction | Implication |
|---|---|---|
| BWT and WWT | Positive | Selecting for heavier weaning weights tends to increase birth weight |
| WWT and PWWT | Strongly positive | Growth traits track together |
| NLB and NWT | Moderate positive | More lambs born generally means more weaned, but not proportionally |
| EMD and FAT | Weakly positive | Muscular animals tend to carry slightly more fat |
| WEC and FEC | Strongly positive | Both measure parasite resistance |
| NLB and lamb survival | Negative | More lambs born per litter means lower individual survival |

### Benefits of Multi-Trait Evaluation

- **Correlated traits inform each other.** Even if weaning weight is not measured for an animal, its birth weight record provides indirect information about weaning weight through the known genetic correlation.
- **Missing data is handled naturally.** Not all traits are recorded for all animals. Multi-trait BLUP uses whatever data is available without discarding incomplete records.
- **Genetic trends are tracked correctly.** Correlated responses to selection (unintended changes in traits not under direct selection) are properly accounted for.

---

## From Raw Data to EBV: The Pipeline

The complete evaluation pipeline involves several stages:

### 1. Data Collection

Breeders record performance data on their animals: birth weights, weaning weights, ultrasound measurements (EMD, FAT), reproduction records (NLB, NWT), fecal egg counts (WEC, FEC), scrotal circumference (SC), and wool traits for wool breeds. This data is submitted to NSIP every two weeks along with pedigree information (sire, dam) and management details (flock, birth date, birth type). The data flows through Pedigree Master software to LAMBPLAN in Australia for BLUP processing.

### 2. Data Validation

Incoming data is checked for errors: impossible values (negative weights, animals born before their parents), missing pedigree links, duplicate records, and outliers. Invalid records are flagged for correction.

### 3. Contemporary Group Formation

Animals are grouped into contemporary groups based on flock, year, season, and management. These groups define the comparison set -- animals are evaluated relative to their contemporaries.

### 4. BLUP Evaluation

The mixed model equations are constructed and solved, incorporating all performance records, pedigree relationships, and (where available) genomic data. This produces EBVs and accuracies for every animal in the evaluation for all 13 traits.

### 5. Publication

Results are published to the NSIP Search API, where they can be accessed through the `nsip` CLI and library. The `date_last_updated` endpoint reports when the most recent evaluation was published.

---

## EBVs and EPDs

EBVs and **Expected Progeny Differences (EPDs)** are closely related concepts. An EPD is simply half the EBV difference between two animals -- because each parent contributes half its genetics to its offspring. Some livestock industries (notably U.S. beef cattle) use EPDs as the primary metric, while NSIP follows the Australian convention of reporting full EBVs. When comparing across systems, remember: EPD = EBV / 2 for a single parent's contribution.

---

## Accuracy: A Measure of Information

Accuracy quantifies how much information backs an EBV estimate. It ranges from 0% (no information) to 100% (perfect knowledge of the true breeding value).

### Factors That Increase Accuracy

| Factor | Effect on Accuracy |
|---|---|
| Own performance records | Moderate increase |
| Progeny records | Large increase per progeny |
| Number of flocks with progeny | Increases by removing flock-environment confounding |
| Genomic data | Moderate increase, especially for young animals |
| Trait heritability | Higher heritability means own records are more informative |
| Pedigree depth and completeness | Better pedigree improves parent average estimates |

### Accuracy and Selection Risk

The practical implication of accuracy is **selection risk**. A ram with EBV +5.0 and 30% accuracy might truly be anywhere from +1.0 to +9.0. The same EBV at 90% accuracy might range from +4.0 to +6.0. Producers pay more for accuracy because it reduces the chance of a disappointing outcome.

In the `nsip` crate, accuracy is stored as `Option<i32>` (integer percentage) in the `Trait` struct. See [Understanding EBVs](EBV-EXPLAINED.md) for a detailed discussion of accuracy interpretation.

---

## Genetic Trend

Over time, selection drives genetic improvement. The breed average EBV for selected traits increases (or decreases for traits where lower is preferred). This genetic trend is a key measure of a breeding program's success.

Because EBVs are expressed relative to a base, a breed undergoing strong selection for growth will see its average WWT EBV increase over the years. An animal that was above average ten years ago might be below average today even though its genetics have not changed -- the rest of the breed has improved around it.

This is why the database update date matters: it tells you which evaluation run your data comes from.

---

## Limitations of the System

No evaluation system is perfect. Key limitations include:

- **Data quality depends on breeders.** Incorrect parentage, inconsistent measurement protocols, or selective reporting (only recording good animals) can bias results.
- **Small breed populations** have fewer data points and fewer connectedness links, leading to lower accuracy and less reliable across-flock comparisons.
- **Genotype-by-environment interaction.** An animal's genetics may express differently in different environments. BLUP assumes the genetic rankings are consistent across environments, which is approximately but not perfectly true.
- **Non-additive genetic effects** (dominance, epistasis) are not captured by standard BLUP. These effects contribute to performance but are not heritable in the same predictable way as additive effects.

---

## Further Reading

- [Understanding EBVs](EBV-EXPLAINED.md) -- interpreting EBV values and accuracy
- [Breed Groups and Traits](BREED-GROUPS-AND-TRAITS.md) -- which traits are evaluated for which breeds
- [NSIP Data Model](NSIP-DATA-MODEL.md) -- how the API structures evaluation results
- [Data to Decisions](DATA-TO-DECISIONS.md) -- applying genetic evaluation to breeding programs
- [How to Compare Animals](../how-to/COMPARE-ANIMALS.md) -- practical comparison techniques

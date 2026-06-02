---
diataxis_type: explanation
---
# Breed Groups and Traits

> How NSIP organizes sheep breeds into evaluation groups, and which EBV traits are relevant to each group.

---

## Why Breed Groups Exist

Sheep breeds vary enormously in their production characteristics. A Rambouillet (fine wool breed) and a Katahdin (hair breed) have fundamentally different selection objectives, trait profiles, and genetic parameters. Evaluating them together would be meaningless because the genetic correlations, heritabilities, and economic weights differ between production types.

NSIP organizes breeds into **breed groups** that share similar production objectives and evaluation frameworks. Within a breed group, the genetic parameters used for BLUP evaluation are appropriate for all member breeds, and cross-breed comparisons (where connectedness exists) are more meaningful.

---

## Breed Group Structure in the API

Breed groups are the entry point to the NSIP data hierarchy. Each group has a numeric ID, a name, and a list of member breeds, and each breed in turn carries its own numeric ID and name. (For the exact Rust types that model this, see [NSIP Data Model](NSIP-DATA-MODEL.md).)

The breed group ID is required for many API operations, particularly searching for animals and querying trait ranges. The breed ID further narrows within a group. These identifiers are what tie the conceptual hierarchy to concrete queries: every search, comparison, and trait-range lookup is ultimately scoped by a breed or breed group ID.

---

## Major Breed Group Categories

NSIP organizes its 23 participating breeds into four primary groups, each reflecting distinct production objectives and evaluation frameworks:

### USA Hair

Shedding breeds that do not require shearing. They are selected primarily for meat production and maternal traits. Katahdin is the most-represented breed in the NSIP system, accounting for roughly a third of all records as of the most recent evaluation cycle.

**Breeds:** Katahdin, Dorper, St. Croix

**Key traits:** BWT, WWT, MWWT, PWWT, YWT, NLB, NWT, EMD, FAT, WEC, FEC, SC

Hair sheep evaluations do not include wool traits. The **USA MAT-HAIR Index** is the primary selection index for this group -- it maximizes total weight of lamb weaned per ewe lambing by combining DWWT, MWWT, NLB, and NWT, with NWT receiving the heaviest economic weighting. In this index context, **DWWT (Direct Weaning Weight) is the same trait as WWT** -- it captures the lamb's own direct genetic contribution to weaning weight. The "Direct" label exists only to distinguish it from MWWT (Maternal Weaning Weight), which captures the dam's contribution through milk and mothering. The two components are reported separately so the index can weight a lamb's own growth and its dam's maternal ability independently.

### USA Terminal

Terminal sires are used in crossbreeding programs. Their offspring are all destined for market (not kept as breeding replacements), so selection focuses on growth and carcass merit.

**Breeds:** Suffolk, Hampshire, Texel, Dorset, White Suffolk, Southdown

**Key traits:** BWT, WWT, PWWT, YWT, EMD, FAT. Maternal traits are less emphasized because daughters are not typically retained.

The **USA Terminal Index** is the primary selection index for this group, emphasizing lean meat production and growth rate.

### USA Maternal

Breeds selected for maternal performance -- the ewe's ability to conceive, carry lambs to term, produce milk, and raise healthy offspring. In practice, these breeds are evaluated for both maternal and growth traits.

**Breeds:** Polypay, Finnsheep, Coopworth, Border Leicester, Corriedale

**Key traits:** NLB, NWT, MWWT, WWT, BWT, WEC, FEC

### USA Range

Western range and wool breeds that produce both meat and wool. Selection balances growth, carcass quality, and fleece characteristics.

**Breeds:** Targhee, Rambouillet, Columbia, SAMM (South African Meat Merino)

**Key traits:** All growth, carcass, and reproduction traits, plus wool traits (GFW, CFW, FD, SL, SS, FDCV, CURV) that are only meaningful for wool-producing breeds.

### Other / Dual Purpose

Several breeds fall outside the four primary groups: Romney, Cheviot, Clun Forest, Shropshire, Tunis, Black Welsh Mountain, and various Composite/Commercial/Terminal entries. These breeds may have more limited trait evaluations depending on data availability.

---

## Trait Availability by Breed

Not every breed has data for every trait. Trait availability depends on:

1. **Relevance.** Wool traits are not measured in hair breeds. WEC/FEC are not routinely measured in all breeds.
2. **Data volume.** A trait must have sufficient performance records to estimate genetic parameters reliably. Small breeds or newly added traits may have limited data.
3. **Recording infrastructure.** Some traits (like ultrasound EMD and FAT) require specialized equipment that not all breeders have access to.

The breed-level trait-ranges data is what reveals, in practice, which traits are available for a specific breed and what value spread to expect. A trait that appears with a meaningful min and max is evaluated for that breed; a trait absent from the response is either not evaluated there or lacks sufficient data to estimate genetic parameters reliably. This makes the trait-ranges view both a catalog of what can be queried and a sanity check on the filters you set. See [How to Filter Search Results](../how-to/FILTER-SEARCH-RESULTS.md) for retrieving and using these ranges.

---

## Understanding the 13 Traits in Context

### Growth Traits: The Foundation

Growth traits (BWT, WWT, MWWT, PWWT, YWT) are evaluated for virtually all breeds because weight gain is a fundamental economic driver in sheep production. All growth EBVs are expressed in **lbs** in the NSIP Search API.

**Birth Weight (BWT)** stands apart from the other growth traits. While heavier weights at weaning and beyond are desirable, heavier birth weights increase the risk of dystocia (lambing difficulty), particularly in first-parity ewes. The genetic correlation between BWT and later growth traits means that selecting for heavier weaning weights tends to increase birth weight as well. This is a key trade-off that selection indexes address by assigning negative economic weight to BWT.

**Weaning Weight (WWT)** is measured at 60 days and is the most widely recorded and economically important growth trait. It reflects the lamb's direct genetic growth potential.

**Maternal Weaning Weight (MWWT)** is a distinct trait from WWT -- it measures the dam's genetic contribution to her lambs' weaning weight through milk production and maternal care. Higher MWWT indicates ewes that raise heavier lambs, making it a key trait for maternal selection.

**Post-Weaning Weight (PWWT) and Yearling Weight (YWT)** measure growth potential beyond weaning, when the lamb is growing on its own nutrition rather than its dam's milk. These traits are particularly important for operations that sell feeder lambs or retain animals to heavier weights.

### Carcass Traits: Meeting Market Specifications

Carcass traits are standardized to a reference body weight of **55 kg (121 lbs)** to allow fair comparison across animals measured at different weights.

**Post-Weaning Eye Muscle Depth (EMD)** measures the cross-sectional area of the longissimus dorsi (loin) muscle via ultrasound scanning, in mm. Higher EMD indicates greater lean meat yield and is almost always desirable.

**Post-Weaning Fat Depth (FAT/CF)** measures subcutaneous fat thickness via ultrasound, in mm. Lower values are preferred, indicating leaner carcasses with better dressing percentage.

NSIP also provides a **Carcass Plus** composite that combines EMD, FAT, and PWWT into a single carcass merit value for simplified selection.

### Reproduction Traits: Profitability Drivers

Reproduction efficiency is the largest single driver of profitability in sheep enterprises. Reproduction traits are expressed as **% above breed average**.

**Number of Lambs Born (NLB)** measures prolificacy. Higher NLB means more lambs per ewe per year. However, very high NLB (triplets and quads) comes with increased lamb mortality, higher labor requirements, and potential animal welfare concerns.

**Number of Lambs Weaned (NWT)** captures the combined effect of prolificacy (NLB) and lamb survival. It is a more complete measure of reproductive success than NLB alone because it includes the ewe's ability to raise her lambs to weaning. NWT receives the heaviest economic weighting in the USA MAT-HAIR Index.

**Scrotal Circumference (SC)** is a male fertility indicator measured in mm. Higher values correlate with improved fertility in both the ram and his daughters, making it valuable for indirect selection on female reproduction.

### Parasite Resistance Traits: Reducing Input Costs

**Weaning Fecal Egg Count (WEC) and Post-Weaning Fecal Egg Count (FEC)** measure resistance to internal parasites, specifically gastrointestinal nematodes. These are expressed as **% relative to breed average**, where **negative values indicate greater resistance**. For example, a ram with a WEC of -90% has the potential to reduce worm burden in his lambs by approximately 45% (since half the genetics pass to offspring).

Selecting for parasite resistance reduces the need for anthelmintic (deworming) treatments, which saves labor and chemical costs, slows the development of drug-resistant parasite populations, and improves animal welfare. These traits are gaining importance as anthelmintic resistance spreads globally.

### Wool Traits (Wool Breeds Only)

For USA Range and other wool-producing breeds, additional traits are evaluated: GFW (Greasy Fleece Weight), CFW (Clean Fleece Weight), FD (Fiber Diameter), SL (Staple Length), SS (Staple Strength), FDCV (Fiber Diameter CV), and CURV (Fiber Curvature). These traits are irrelevant for hair breeds.

---

## Trait Interactions and Trade-offs

Understanding trait correlations is essential for effective selection. The major interactions include:

### Positive Correlations (Selecting for One Increases the Other)

- BWT with WWT, PWWT, YWT -- growth genes tend to affect all stages
- WWT with PWWT -- early and late growth are strongly linked
- NLB with NWT -- more born usually means more weaned
- WEC with FEC -- both measure aspects of parasite resistance

### Antagonistic Relationships (Trade-offs)

- BWT with ease of lambing -- heavier lambs are harder to deliver
- NLB with individual lamb survival -- larger litters have higher per-lamb mortality
- Lean growth (EMD) with fat coverage -- pushing for extreme leanness reduces fat cover
- Growth rate with mature size -- faster-growing animals tend to reach larger mature weights, increasing maintenance feed costs for breeding ewes

### Independent Traits

Some trait pairs are largely independent, meaning selection on one has minimal effect on the other. These represent opportunities to improve multiple traits simultaneously without trade-offs.

---

## Choosing Traits for Your Breeding Objective

The appropriate traits to select depend on your production system and market:

| Production System | Priority Traits | Secondary Traits | Recommended Index |
|---|---|---|---|
| Terminal sire (all lambs marketed) | WWT, PWWT, EMD, FAT | BWT (minimize) | USA Terminal Index |
| Self-replacing hair flock | NWT, MWWT, WWT | BWT, WEC | USA MAT-HAIR Index |
| Dual-purpose (meat + wool) | WWT, NWT, GFW, FD | EMD, FAT | -- |
| Parasite-challenged environment | WEC/FEC, NWT | WWT, MWWT | -- |
| Low-input/extensive | NWT, MWWT | BWT (minimize), WEC | USA MAT-HAIR Index |

Published selection indexes combine these traits with appropriate economic weights. See [Understanding EBVs](EBV-EXPLAINED.md) for more on selection indexes.

---

## Putting Breed and Trait Knowledge to Use

Knowing how breeds are grouped and which traits each group is evaluated for is the foundation for two practical tasks: discovering the value ranges a breed actually spans, and searching within a breed for animals that fit your objective. For the commands and code that list breeds, retrieve a breed's trait ranges, and filter a search within a breed group, see [How to Filter Search Results](../how-to/FILTER-SEARCH-RESULTS.md) and [How to Compare Animals](../how-to/COMPARE-ANIMALS.md).

---

## Further Reading

- [Understanding EBVs](EBV-EXPLAINED.md) -- interpreting EBV values and selection indexes
- [Genetic Evaluation](GENETIC-EVALUATION.md) -- how BLUP produces breed-specific evaluations
- [NSIP Data Model](NSIP-DATA-MODEL.md) -- the data structures behind breed groups and traits
- [Data to Decisions](DATA-TO-DECISIONS.md) -- applying breed and trait knowledge to selection
- [How to Compare Animals](../how-to/COMPARE-ANIMALS.md) -- comparing animals within a breed

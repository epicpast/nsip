# Breed Groups and Traits

> How NSIP organizes sheep breeds into evaluation groups, and which EBV traits are relevant to each group.

---

## Why Breed Groups Exist

Sheep breeds vary enormously in their production characteristics. A Rambouillet (fine wool breed) and a Katahdin (hair breed) have fundamentally different selection objectives, trait profiles, and genetic parameters. Evaluating them together would be meaningless because the genetic correlations, heritabilities, and economic weights differ between production types.

NSIP organizes breeds into **breed groups** that share similar production objectives and evaluation frameworks. Within a breed group, the genetic parameters used for BLUP evaluation are appropriate for all member breeds, and cross-breed comparisons (where connectedness exists) are more meaningful.

---

## Breed Group Structure in the API

Breed groups are the entry point to the NSIP data hierarchy. Each group has a numeric ID, a name, and a list of member breeds.

```rust
pub struct BreedGroup {
    pub id: i64,
    pub name: String,
    pub breeds: Vec<Breed>,
}

pub struct Breed {
    pub id: i64,
    pub name: String,
}
```

The breed group ID is required for many API operations, particularly searching for animals and querying trait ranges. The breed ID further narrows within a group.

```bash
# List all breed groups
nsip breed-groups

# Get trait ranges for a specific breed
nsip trait-ranges 640
```

---

## Major Breed Group Categories

NSIP organizes its 23 participating breeds into four primary groups, each reflecting distinct production objectives and evaluation frameworks:

### USA Hair

Shedding breeds that do not require shearing. They are selected primarily for meat production and maternal traits. Katahdin is the most-represented breed in the NSIP system, accounting for approximately 35% of all records.

**Breeds:** Katahdin, Dorper, St. Croix

**Key traits:** BWT, WWT, MWWT, PWWT, YWT, NLB, NLW, PEMD, PFAT, WFEC, PFEC, SC

Hair sheep evaluations do not include wool traits. The **USA MAT-HAIR Index** is the primary selection index for this group -- it maximizes total weight of lamb weaned per ewe lambing by combining DWWT, MWWT, NLB, and NLW, with NLW receiving the heaviest economic weighting.

### USA Terminal

Terminal sires are used in crossbreeding programs. Their offspring are all destined for market (not kept as breeding replacements), so selection focuses on growth and carcass merit.

**Breeds:** Suffolk, Hampshire, Texel, Dorset, White Suffolk, Southdown

**Key traits:** BWT, WWT, PWWT, YWT, PEMD, PFAT. Maternal traits are less emphasized because daughters are not typically retained.

The **USA Terminal Index** is the primary selection index for this group, emphasizing lean meat production and growth rate.

### USA Maternal

Breeds selected for maternal performance -- the ewe's ability to conceive, carry lambs to term, produce milk, and raise healthy offspring. In practice, these breeds are evaluated for both maternal and growth traits.

**Breeds:** Polypay, Finnsheep, Coopworth, Border Leicester, Corriedale

**Key traits:** NLB, NLW, MWWT, WWT, BWT, WFEC, PFEC

### USA Range

Western range and wool breeds that produce both meat and wool. Selection balances growth, carcass quality, and fleece characteristics.

**Breeds:** Targhee, Rambouillet, Columbia, SAMM (South African Meat Merino)

**Key traits:** All growth, carcass, and reproduction traits, plus wool traits (GFW, CFW, FD, SL, SS, FDCV, CURV) that are only meaningful for wool-producing breeds.

### Other / Dual Purpose

Several breeds fall outside the four primary groups: Romney, Cheviot, Clun Forest, Shropshire, Tunis, Black Welsh Mountain, and various Composite/Commercial/Terminal entries. These breeds may have more limited trait evaluations depending on data availability.

---

## Trait Availability by Breed

Not every breed has data for every trait. Trait availability depends on:

1. **Relevance.** Wool traits are not measured in hair breeds. WFEC/PFEC are not routinely measured in all breeds.
2. **Data volume.** A trait must have sufficient performance records to estimate genetic parameters reliably. Small breeds or newly added traits may have limited data.
3. **Recording infrastructure.** Some traits (like ultrasound EMD and FAT) require specialized equipment that not all breeders have access to.

Use the `trait_ranges` endpoint to discover which traits are available for a specific breed and what value ranges to expect:

```rust
let ranges = client.trait_ranges(breed_id).await?;
for range in &ranges {
    println!("{}: {:.2} to {:.2} {}",
        range.trait_name,
        range.min_value,
        range.max_value,
        range.unit.as_deref().unwrap_or(""),
    );
}
```

If a trait is not returned by `trait_ranges` for a given breed, that trait is either not evaluated for that breed or has insufficient data.

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

**Post-Weaning Eye Muscle Depth (PEMD/EMD)** measures the cross-sectional area of the longissimus dorsi (loin) muscle via ultrasound scanning, in mm. Higher PEMD indicates greater lean meat yield and is almost always desirable.

**Post-Weaning Fat Depth (PFAT/CF)** measures subcutaneous fat thickness via ultrasound, in mm. Lower values are preferred, indicating leaner carcasses with better dressing percentage.

NSIP also provides a **Carcass Plus** composite that combines EMD, FAT, and PWWT into a single carcass merit value for simplified selection.

### Reproduction Traits: Profitability Drivers

Reproduction efficiency is the largest single driver of profitability in sheep enterprises. Reproduction traits are expressed as **% above breed average**.

**Number of Lambs Born (NLB)** measures prolificacy. Higher NLB means more lambs per ewe per year. However, very high NLB (triplets and quads) comes with increased lamb mortality, higher labor requirements, and potential animal welfare concerns.

**Number of Lambs Weaned (NLW)** captures the combined effect of prolificacy (NLB) and lamb survival. It is a more complete measure of reproductive success than NLB alone because it includes the ewe's ability to raise her lambs to weaning. NLW receives the heaviest economic weighting in the USA MAT-HAIR Index.

**Scrotal Circumference (SC)** is a male fertility indicator measured in mm. Higher values correlate with improved fertility in both the ram and his daughters, making it valuable for indirect selection on female reproduction.

### Parasite Resistance Traits: Reducing Input Costs

**Weaning Fecal Egg Count (WFEC) and Post-Weaning Fecal Egg Count (PFEC)** measure resistance to internal parasites, specifically gastrointestinal nematodes. These are expressed as **% relative to breed average**, where **negative values indicate greater resistance**. For example, a ram with a WFEC of -90% has the potential to reduce worm burden in his lambs by approximately 45% (since half the genetics pass to offspring).

Selecting for parasite resistance reduces the need for anthelmintic (deworming) treatments, which saves labor and chemical costs, slows the development of drug-resistant parasite populations, and improves animal welfare. These traits are gaining importance as anthelmintic resistance spreads globally.

### Wool Traits (Wool Breeds Only)

For USA Range and other wool-producing breeds, additional traits are evaluated: GFW (Greasy Fleece Weight), CFW (Clean Fleece Weight), FD (Fiber Diameter), SL (Staple Length), SS (Staple Strength), FDCV (Fiber Diameter CV), and CURV (Fiber Curvature). These traits are irrelevant for hair breeds.

---

## Trait Interactions and Trade-offs

Understanding trait correlations is essential for effective selection. The major interactions include:

### Positive Correlations (Selecting for One Increases the Other)

- BWT with WWT, PWWT, YWT -- growth genes tend to affect all stages
- WWT with PWWT -- early and late growth are strongly linked
- NLB with NLW -- more born usually means more weaned
- WFEC with PFEC -- both measure aspects of parasite resistance

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
| Terminal sire (all lambs marketed) | WWT, PWWT, PEMD, PFAT | BWT (minimize) | USA Terminal Index |
| Self-replacing hair flock | NLW, MWWT, WWT | BWT, WFEC | USA MAT-HAIR Index |
| Dual-purpose (meat + wool) | WWT, NLW, GFW, FD | PEMD, PFAT | -- |
| Parasite-challenged environment | WFEC/PFEC, NLW | WWT, MWWT | -- |
| Low-input/extensive | NLW, MWWT | BWT (minimize), WFEC | USA MAT-HAIR Index |

Published selection indexes combine these traits with appropriate economic weights. See [Understanding EBVs](EBV-EXPLAINED.md) for more on selection indexes.

---

## Querying Breed and Trait Data

### List All Available Breeds

```bash
nsip breed-groups
```

```rust
let groups = client.breed_groups().await?;
```

### Get Trait Ranges for a Breed

```bash
nsip trait-ranges <breed_id>
```

```rust
let ranges = client.trait_ranges(breed_id).await?;
```

### Search Within a Breed Group

```bash
nsip search --breed-id 640 --status CURRENT --gender Male
```

```rust
let criteria = SearchCriteria::new()
    .with_breed_id(640)
    .with_status("CURRENT")
    .with_gender("Male");

let results = client.search_animals(0, 25, 640, None, false, &criteria).await?;
```

---

## Further Reading

- [Understanding EBVs](EBV-EXPLAINED.md) -- interpreting EBV values and selection indexes
- [Genetic Evaluation](GENETIC-EVALUATION.md) -- how BLUP produces breed-specific evaluations
- [NSIP Data Model](NSIP-DATA-MODEL.md) -- the data structures behind breed groups and traits
- [Data to Decisions](DATA-TO-DECISIONS.md) -- applying breed and trait knowledge to selection
- [How to Compare Animals](../how-to/COMPARE-ANIMALS.md) -- comparing animals within a breed

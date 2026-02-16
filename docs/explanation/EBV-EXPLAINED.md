# Understanding EBVs (Estimated Breeding Values)

> **Goal:** Understand what EBVs are, how they're calculated, and how to use them for breeding decisions.

---

## What is an EBV?

An **Estimated Breeding Value** (EBV) predicts an animal's genetic merit for a specific trait. It estimates the value an animal will pass to its offspring, not the animal's own performance.

**Key points:**
- EBVs are **relative to a breed average** (typically set to 0)
- **Positive EBV** = better than average genetics
- **Negative EBV** = worse than average genetics
- **EBV units** vary by trait (kg, mm, days, etc.)

---

## Example: Weight EBV

If a ram has a **Weight EBV of +8.5 kg**:
- Its offspring will be **8.5 kg heavier** than average (all else equal)
- The breed average is **0 kg**
- A ram with **-3.2 kg** would produce lighter offspring

---

## Accuracy

Every EBV includes an **accuracy** value (0.0 to 1.0):

| Accuracy | Interpretation | Confidence Level |
|----------|----------------|------------------|
| 0.00-0.29 | Low | Minimal data, high uncertainty |
| 0.30-0.59 | Medium | Some progeny or genomic data |
| 0.60-0.79 | High | Good progeny records |
| 0.80-1.00 | Very High | Extensive progeny data |

**Rule of thumb:** Prefer animals with **accuracy ≥ 0.60** for important breeding decisions.

**Example:**
````rust
let weight_trait = animal.traits.get("Weight").unwrap();
println!("Weight EBV: {} (accuracy: {})", weight_trait.value, weight_trait.accuracy);

if weight_trait.accuracy < 0.60 {
    println!("⚠️  Low accuracy - use with caution");
}
````

---

## Common Trait Categories

### Production Traits
- **Weight** - Growth rate and mature size
- **Muscle** - Meat yield and lean meat percentage
- **Fat** - Carcass fat coverage (lower is often better)

### Maternal Traits
- **Milk** - Maternal milk production affecting offspring growth
- **Fertility** - Conception rate and lambing percentage
- **Litter Size** - Number of lambs per litter

### Survival Traits
- **Survival to Weaning** - Lamb viability
- **Longevity** - Productive lifespan

### Carcass Traits
- **Eye Muscle Depth** - Loin muscle size
- **Fat Depth** - Subcutaneous fat thickness

---

## Genetic Trend

EBVs are **adjusted over time** as the breed average improves:

- A ram with +10 kg in 2020 might be average (+0 kg) in 2030
- Always compare animals from the **same evaluation year**
- NSIP provides **Last Updated** dates for the database

Check the database date:
````bash
nsip date-updated
````

Or programmatically:
````rust
let date_info = client.date_last_updated().await?;
println!("Database last updated: {}", date_info.last_updated_date);
````

---

## Selection Indexes

Most breeding programs use **selection indexes** that combine multiple EBVs into a single value:

| Index | Focus | Typical Traits |
|-------|-------|----------------|
| **Terminal Index** | Meat production | Weight, Muscle, Fat |
| **Maternal Index** | Reproductive performance | Milk, Fertility, Litter Size |
| **Dual Purpose Index** | Balanced production | Weight, Muscle, Milk, Fertility |

**Example:** A Terminal Index might be:
````
Index = (0.4 × Weight EBV) + (0.4 × Muscle EBV) - (0.2 × Fat EBV)
````

---

## How NSIP Calculates EBVs

1. **Pedigree data** - Ancestry relationships (sire, dam, grandparents)
2. **Performance records** - Measured traits (birth weight, weaning weight, etc.)
3. **Genomic data** (if available) - DNA markers for trait prediction
4. **BLUP algorithm** - Best Linear Unbiased Prediction statistical model

The calculation accounts for:
- **Genetic relationships** between animals
- **Environmental effects** (year, flock, management)
- **Fixed effects** (age, sex, litter size)

---

## Using EBVs in Practice

### Single Trait Selection

Select the top animal for one trait:

````rust
let mut animals = search_results.results;
animals.sort_by(|a, b| {
    let a_weight = a.traits.get("Weight").map(|t| t.value).unwrap_or(f64::NEG_INFINITY);
    let b_weight = b.traits.get("Weight").map(|t| t.value).unwrap_or(f64::NEG_INFINITY);
    b_weight.partial_cmp(&a_weight).unwrap()
});

println!("Top animal: {} (Weight EBV: {})", 
    animals[0].lpn_id, 
    animals[0].traits["Weight"].value
);
````

### Multi-Trait Selection

Use weighted trait scores (see [How to Rank Animals](../how-to/RANK-ANIMALS.md)).

### Avoiding Inbreeding

Check coefficient of inbreeding (COI) before mating:

````bash
nsip mcp
# Then use the inbreeding_check tool
````

---

## Trait Ranges by Breed

Different breeds have different trait ranges. Get valid ranges:

````bash
nsip trait-ranges 640  # Maternal breed group
````

Or programmatically:
````rust
let ranges = client.trait_ranges(640).await?;
for range in ranges {
    println!("{}: min={}, max={}", range.name, range.min, range.max);
}
````

---

## Common Mistakes

❌ **Comparing EBVs across breeds** - EBVs are breed-specific  
❌ **Ignoring accuracy** - High EBV with low accuracy is risky  
❌ **Selecting on one trait only** - Causes unintended changes in other traits  
❌ **Using outdated data** - Always check database last-updated date  

✅ **Use breed-specific comparisons**  
✅ **Weight decisions by accuracy**  
✅ **Balance multiple traits via indexes**  
✅ **Verify data currency**  

---

## Further Reading

- [How to Compare Animals](../how-to/COMPARE-ANIMALS.md) - Practical comparison techniques
- [How to Rank Animals](../how-to/RANK-ANIMALS.md) - Multi-trait ranking
- [NSIP Official Documentation](https://www.nsip.org/) - Industry standards
- [Sheep Genetics 101](https://sheepgenetics.org.au/) - Educational resources

---

## See Also

- [Inbreeding Coefficient Explained](INBREEDING-EXPLAINED.md)
- [Genomic Selection](GENOMIC-SELECTION.md)
- [Breeding Program Design](BREEDING-PROGRAMS.md)

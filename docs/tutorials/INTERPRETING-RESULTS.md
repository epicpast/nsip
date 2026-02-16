# Interpreting Results

> **Learning Goal:** By the end of this tutorial, you will understand how to read the data returned by the NSIP API -- what EBV values mean, how to assess accuracy, how to read lineage data, and how to make sense of an animal's complete profile.

**Time to complete:** 10 minutes
**Prerequisites:**
- Completed the [Getting Started](GETTING-STARTED.md) tutorial
- Familiarity with the basic API calls (`animal_details`, `search_by_lpn`)

---

## What You Will Learn

1. How to read EBV trait values and their units
2. What accuracy means and why it matters
3. How to interpret lineage (pedigree) data
4. How to read progeny records
5. How to put it all together with a complete animal profile

---

## Step 1: Understanding EBV Traits

Fetch an animal's details and examine its traits:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();
    let details = client.animal_details("400001").await?;

    println!("Traits for {}:\n", details.lpn_id);

    for (abbrev, trait_data) in &details.traits {
        println!("  {} ({})", abbrev, trait_data.name);
        println!("    Value: {:.2}", trait_data.value);
        if let Some(acc) = trait_data.accuracy {
            println!("    Accuracy: {}%", acc);
        }
        if let Some(units) = &trait_data.units {
            println!("    Units: {}", units);
        }
        println!();
    }

    Ok(())
}
```

You might see output like:

```
Traits for 400001:

  BWT (Birth Weight)
    Value: 0.35
    Accuracy: 72%
    Units: lbs

  WWT (Weaning Weight)
    Value: 4.20
    Accuracy: 65%
    Units: lbs

  NLB (Number Lambs Born)
    Value: 0.12
    Accuracy: 45%
    Units: %
```

**What do these numbers mean?**

An EBV (Estimated Breeding Value) is not the animal's own weight or measurement. It is a prediction of the **genetic merit** the animal will pass to its offspring, expressed as a deviation from the breed average.

- **BWT = 0.35 lbs** means this animal's offspring are expected to be 0.35 lbs heavier at birth than the breed average. A small positive BWT is typical; very high BWT can indicate lambing difficulty.
- **WWT = 4.20 lbs** means offspring are expected to weigh 4.20 lbs more at weaning (60 days) than average. Higher is generally better for growth traits.
- **NLB = 0.12** means this animal's genetics predict 0.12% more lambs born per lambing than the breed average. This is a reproductive trait expressed as a percentage.

The common EBV traits in the NSIP system are:

**Growth traits:**

| Abbreviation | Trait | Units | Higher means... |
|-------------|-------|-------|-----------------|
| BWT | Birth Weight | lbs | Heavier lambs at birth |
| WWT | Weaning Weight (60 days) | lbs | Faster early growth |
| MWWT | Maternal Weaning Weight | lbs | Better mothering ability |
| PWWT | Post-Weaning Weight | lbs | Faster later growth |
| YWT | Yearling Weight | lbs | Heavier at one year |

**Carcass traits** (standardized to 55 kg / 121 lbs body weight):

| Abbreviation | Trait | Units | Higher means... |
|-------------|-------|-------|-----------------|
| PEMD (EMD) | Eye Muscle Depth | mm | More muscle |
| PFAT (CF) | Fat Depth | mm | More fat cover |

**Reproduction traits:**

| Abbreviation | Trait | Units | Higher means... |
|-------------|-------|-------|-----------------|
| NLB | Number Lambs Born | % | More prolific |
| NLW | Number Lambs Weaned | % | Better lamb survival |
| SC | Scrotal Circumference | mm | Higher fertility (rams) |

**Parasite resistance traits:**

| Abbreviation | Trait | Units | Higher means... |
|-------------|-------|-------|-----------------|
| WFEC | Weaning Fecal Egg Count | % | More worm eggs (undesirable) |
| PFEC | Post-Weaning Fecal Egg Count | % | More parasites (undesirable) |

For parasite traits (WFEC, PFEC), **negative** values are desirable -- they indicate genetic resistance to internal parasites. For example, a ram with a WFEC EBV of -90% would be expected to reduce worm burden in his lambs by approximately 45% (half the EBV, since each parent contributes half).

Wool breeds may also have additional traits such as GFW (Greasy Fleece Weight), CFW (Clean Fleece Weight), FD (Fibre Diameter), SL (Staple Length), SS (Staple Strength), FDCV (Fibre Diameter CV), and CURV (Curvature).

---

## Step 2: Assessing Accuracy

The accuracy value tells you how reliable the EBV estimate is:

```
Accuracy: 72%   -- fairly reliable
Accuracy: 45%   -- use with caution
Accuracy: 90%   -- very reliable
```

Accuracy ranges from 0 to 100. It increases as more data becomes available:

| Accuracy Range | Meaning | Typical Source |
|---------------|---------|----------------|
| 0-29% | Low confidence | Pedigree data only |
| 30-59% | Moderate confidence | Some progeny records or genomic data |
| 60-79% | Good confidence | Multiple progeny records |
| 80-99% | High confidence | Extensive progeny and genomic data |

**Practical guidance:**

- For important breeding decisions (selecting sires), prefer animals with accuracy of 60% or higher on your priority traits.
- Low-accuracy EBVs can change significantly as more data comes in. Treat them as rough estimates.
- A high accuracy on a mediocre EBV is more informative than a low accuracy on an excellent EBV.

---

## Step 3: Reading Lineage Data

Lineage shows an animal's pedigree -- its parents, grandparents, and further ancestors:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();
    let lineage = client.lineage("400001").await?;

    // The subject animal
    if let Some(subject) = &lineage.subject {
        println!("Subject: {} (born {})",
            subject.lpn_id,
            subject.date_of_birth.as_deref().unwrap_or("unknown"),
        );
    }

    // Parents
    if let Some(sire) = &lineage.sire {
        println!("Sire (father): {}", sire.lpn_id);
        if let Some(idx) = sire.us_index {
            println!("  US Index: {:.2}", idx);
        }
    }

    if let Some(dam) = &lineage.dam {
        println!("Dam (mother): {}", dam.lpn_id);
        if let Some(idx) = dam.us_index {
            println!("  US Index: {:.2}", idx);
        }
    }

    // Further generations
    for (gen_num, generation) in lineage.generations.iter().enumerate() {
        println!("\nGeneration {} ({} ancestors):", gen_num + 1, generation.len());
        for ancestor in generation {
            print!("  {}", ancestor.lpn_id);
            if let Some(sex) = &ancestor.sex {
                print!(" ({})", sex);
            }
            if let Some(status) = &ancestor.status {
                print!(" [{}]", status);
            }
            println!();
        }
    }

    Ok(())
}
```

**What to look for in lineage data:**

- **Selection indexes** on ancestors (such as the USA MAT-HAIR Index for hair sheep or the USA Terminal Index for terminal sires) indicate their overall genetic merit. These are composite indexes that combine multiple EBV traits into a single dollar-value score.
- **Status** tells you if ancestors are still `CURRENT`, `SOLD`, or `DEAD`. Dead ancestors still contribute valuable pedigree information.
- **Depth of pedigree** -- more generations of known ancestry generally means more accurate EBV estimates for the subject animal.

---

## Step 4: Reading Progeny Records

Progeny data shows an animal's offspring and their traits:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();
    let progeny = client.progeny("400001", 0, 10).await?;

    println!("Total offspring: {}\n", progeny.total_count);

    for offspring in &progeny.animals {
        print!("{}", offspring.lpn_id);
        if let Some(sex) = &offspring.sex {
            print!(" ({})", sex);
        }
        if let Some(dob) = &offspring.date_of_birth {
            print!(" born {}", dob);
        }
        println!();

        // Display offspring trait values
        for (trait_name, value) in &offspring.traits {
            println!("  {} = {:.2}", trait_name, value);
        }
        println!();
    }

    Ok(())
}
```

**What to look for in progeny data:**

- **Number of offspring** -- more progeny means the parent's EBV estimates are more accurate.
- **Trait consistency** -- if most offspring show similar trait values, the parent is a reliable transmitter of those genetics.
- **Sex distribution** -- relevant for reproductive traits (NLB, NLW) which are primarily expressed in female offspring.
- Progeny data is paginated. Use the `page` and `page_size` parameters to retrieve more offspring.

---

## Step 5: Putting It All Together

The `search_by_lpn` method returns a complete `AnimalProfile` that combines details, lineage, and progeny. Here is how to evaluate an animal holistically:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();
    let profile = client.search_by_lpn("400001").await?;

    // 1. Basic identification
    println!("=== Animal Profile ===\n");
    println!("LPN ID: {}", profile.details.lpn_id);
    if let Some(breed) = &profile.details.breed {
        println!("Breed: {}", breed);
    }
    if let Some(gender) = &profile.details.gender {
        println!("Gender: {}", gender);
    }
    if let Some(status) = &profile.details.status {
        println!("Status: {}", status);
    }

    // 2. Key growth traits
    println!("\n--- Growth Traits ---");
    let growth_traits = ["BWT", "WWT", "PWWT", "YWT"];
    for abbrev in &growth_traits {
        if let Some(t) = profile.details.traits.get(*abbrev) {
            let acc_str = t.accuracy
                .map(|a| format!(" (acc: {}%)", a))
                .unwrap_or_default();
            let units_str = t.units.as_deref().unwrap_or("");
            println!("  {} = {:.2} {}{}", abbrev, t.value, units_str, acc_str);
        }
    }

    // 3. Reproduction traits
    println!("\n--- Reproduction Traits ---");
    let repro_traits = ["NLB", "NLW"];
    for abbrev in &repro_traits {
        if let Some(t) = profile.details.traits.get(*abbrev) {
            let acc_str = t.accuracy
                .map(|a| format!(" (acc: {}%)", a))
                .unwrap_or_default();
            println!("  {} = {:.2}{}", abbrev, t.value, acc_str);
        }
    }

    // 4. Parentage
    println!("\n--- Parentage ---");
    if let Some(sire) = &profile.lineage.sire {
        println!("  Sire: {}", sire.lpn_id);
    }
    if let Some(dam) = &profile.lineage.dam {
        println!("  Dam: {}", dam.lpn_id);
    }

    // 5. Progeny summary
    println!("\n--- Progeny ---");
    println!("  Total offspring: {}", profile.progeny.total_count);

    Ok(())
}
```

**How to evaluate an animal:**

1. **Check status** -- only `CURRENT` animals are actively evaluated and available for breeding.
2. **Prioritize high-accuracy traits** -- focus on traits with accuracy above 60% for breeding decisions.
3. **Consider the breeding objective** -- terminal sire breeds (Suffolk, Hampshire, Texel) prioritize growth and carcass traits (WWT, YWT, PEMD); maternal breeds (Polypay, Finnsheep) prioritize reproduction (NLB, NLW) and lamb survival; hair sheep breeds (Katahdin, Dorper) use the USA MAT-HAIR Index which combines growth and maternal traits.
4. **Look at the pedigree** -- strong ancestors with high indexes suggest the animal's genetics are well-supported.
5. **Check progeny count** -- animals with more offspring have more reliable EBVs.

---

## What You Learned

In this tutorial you:

- Read and interpreted EBV trait values and their units
- Understood what accuracy percentages mean and how to use them
- Explored lineage data including parent and grandparent records
- Read progeny records and trait transmission data
- Combined all data points to evaluate an animal holistically

---

## Next Steps

For deeper understanding:

- [Understanding EBVs](../explanation/EBV-EXPLAINED.md) -- the theory behind Estimated Breeding Values
- [How to Compare Animals](../how-to/COMPARE-ANIMALS.md) -- side-by-side comparison of multiple animals
- [MCP Server Setup](MCP-SERVER-SETUP.md) -- use AI assistants to query and analyze NSIP data interactively

For reference:

- [Error Handling Reference](../reference/ERROR-HANDLING.md) -- handle edge cases in API responses
- [MCP Server Reference](../MCP.md) -- complete API and tool documentation

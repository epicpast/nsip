# How to Export Data as JSON

> **Problem:** You need to export NSIP animal data in JSON format for further processing, reporting, or integration with other tools.

**Prerequisites:**
- `nsip` CLI installed, or `nsip` crate added to your `Cargo.toml`

---

## CLI Method

### Step 1: Add the JSON Flag

Every CLI command supports the `--json` (or `-J`) global flag. Add it to any command to switch from human-readable ASCII tables to JSON output:

```bash
nsip details 430735-0032 -J
```

### Step 2: Export Search Results

```bash
nsip search --breed-id 486 --status CURRENT --sort-by WWT --reverse -J
```

Output is a JSON object with `total_count`, `page`, `page_size`, and a `results` array.

### Step 3: Export Animal Profiles

```bash
nsip profile 430735-0032 -J
```

Returns details, lineage, and progeny as a single JSON object.

### Step 4: Export Breed Groups

```bash
nsip breed-groups -J
```

### Step 5: Export Trait Ranges

```bash
nsip trait-ranges 486 -J
```

### Step 6: Save to a File

Redirect output to a file:

```bash
nsip search --breed-id 486 --status CURRENT -J > current_animals.json
```

### Step 7: Pipe to Other Tools

Combine with `jq` for extraction and transformation:

```bash
# Extract just the LPN IDs from search results
nsip search --breed-id 486 --status CURRENT -J | jq '.results[].lpnId'

# Get a specific trait value from animal details
nsip details 430735-0032 -J | jq '.traits.WWT'

# Pretty-print the output
nsip profile 430735-0032 -J | jq .
```

---

## Library Method

### Step 1: Serialize with serde_json

All NSIP data types implement `Serialize`. Use `serde_json` to convert them:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();
    let details = client.animal_details("430735-0032").await?;

    let json = serde_json::to_string_pretty(&details)
        .expect("serialization should not fail");
    println!("{json}");

    Ok(())
}
```

### Step 2: Export Search Results

```rust
use nsip::{NsipClient, SearchCriteria};

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let criteria = SearchCriteria::new()
        .with_breed_id(486)
        .with_status("CURRENT");

    let results = client
        .search_animals(0, 100, Some(486), Some("WWT"), Some(true), Some(&criteria))
        .await?;

    let json = serde_json::to_string_pretty(&results)
        .expect("serialization should not fail");
    println!("{json}");

    Ok(())
}
```

### Step 3: Write to a File

```rust
use std::fs::File;
use std::io::BufWriter;
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = NsipClient::new();
    let details = client.animal_details("430735-0032").await?;

    let file = File::create("animal_details.json")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &details)?;

    Ok(())
}
```

### Step 4: Export Multiple Animals

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();
    let ids = ["430735-0032", "430735-0041", "430735-0058"];

    let mut animals = Vec::new();
    for id in &ids {
        let details = client.animal_details(id).await?;
        animals.push(details);
    }

    let json = serde_json::to_string_pretty(&animals)
        .expect("serialization should not fail");
    println!("{json}");

    Ok(())
}
```

---

## Verify the Output

1. Validate the JSON structure with `jq`:

   ```bash
   nsip details 430735-0032 -J | jq type
   # Should output: "object"
   ```

2. Check that expected fields are present:

   ```bash
   nsip details 430735-0032 -J | jq 'keys'
   ```

3. Verify the file is valid JSON if saved:

   ```bash
   jq . < animal_details.json > /dev/null && echo "Valid JSON"
   ```

---

## See Also

- [How to Filter Search Results](FILTER-SEARCH-RESULTS.md) -- filter before exporting
- [How to Integrate with Scripts](SCRIPTING-INTEGRATION.md) -- use JSON in automation pipelines
- [How to Batch Query Multiple Animals](BATCH-QUERY.md)

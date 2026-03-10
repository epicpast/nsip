---
diataxis_type: how-to
---
# How to Batch Query Multiple Animals

> **Problem:** You need to retrieve data for many animals at once, either from a list of LPN IDs or by paginating through search results.

**Prerequisites:**
- `nsip` CLI installed, or `nsip` crate added to your `Cargo.toml`
- A list of LPN IDs or search criteria to identify the animals

---

## CLI Method

### Step 1: Query Multiple Animals from a List

Use a shell loop to iterate over LPN IDs:

```bash
for id in 430735-0032 430735-0041 430735-0058; do
    nsip details "$id" -J
done
```

### Step 2: Query from a File

If you have LPN IDs in a file (one per line):

```bash
while IFS= read -r id; do
    nsip details "$id" -J
done < lpn_ids.txt
```

### Step 3: Collect Results into a JSON Array

Use `jq` to combine individual results into an array:

```bash
while IFS= read -r id; do
    nsip details "$id" -J
done < lpn_ids.txt | jq -s '.'
```

### Step 4: Paginate Through Search Results

Fetch all pages of a search:

```bash
page=0
while true; do
    result=$(nsip search --breed-id 486 --status CURRENT --page-size 100 -p "$page" -J)
    echo "$result" | jq '.results[]'

    total=$(echo "$result" | jq '.total_count')
    fetched=$(( (page + 1) * 100 ))
    if [ "$fetched" -ge "$total" ]; then
        break
    fi
    page=$((page + 1))
done
```

---

## Library Method

### Step 1: Fetch Multiple Animals Concurrently

Use `tokio::join!` for a small, known set of animals:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let (a, b, c) = tokio::join!(
        client.animal_details("430735-0032"),
        client.animal_details("430735-0041"),
        client.animal_details("430735-0058"),
    );

    let animals = vec![a?, b?, c?];

    for animal in &animals {
        println!("{}: {:?}", animal.lpn_id, animal.breed);
    }

    Ok(())
}
```

### Step 2: Fetch a Dynamic List with Controlled Concurrency

For larger lists, process animals in batches to avoid overwhelming the API:

```rust
use nsip::{AnimalDetails, NsipClient};

async fn fetch_batch(
    client: &NsipClient,
    ids: &[&str],
    batch_size: usize,
) -> Vec<Result<AnimalDetails, nsip::Error>> {
    let mut results = Vec::new();

    for chunk in ids.chunks(batch_size) {
        let mut handles = Vec::new();
        for id in chunk {
            handles.push(client.animal_details(id));
        }

        // Await all futures in this batch
        for handle in handles {
            results.push(handle.await);
        }
    }

    results
}

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();
    let ids = vec!["430735-0032", "430735-0041", "430735-0058"];

    let results = fetch_batch(&client, &ids, 5).await;

    for result in results {
        match result {
            Ok(animal) => println!("{}: {:?}", animal.lpn_id, animal.breed),
            Err(e) => eprintln!("Error: {e}"),
        }
    }

    Ok(())
}
```

### Step 3: Paginate Through All Search Results

Collect all animals matching search criteria across multiple pages:

```rust
use nsip::{NsipClient, SearchCriteria};

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();

    let criteria = SearchCriteria::new()
        .with_breed_id(486)
        .with_status("CURRENT");

    let page_size = 100;
    let mut page = 0;
    let mut all_results = Vec::new();

    loop {
        let results = client
            .search_animals(page, page_size, Some(486), None, None, Some(&criteria))
            .await?;

        all_results.extend(results.results);

        if all_results.len() as i64 >= results.total_count {
            break;
        }
        page += 1;
    }

    println!("Fetched {} total animals", all_results.len());

    Ok(())
}
```

### Step 4: Fetch Full Profiles in Batch

Use `search_by_lpn()` to get details, lineage, and progeny in a single call per animal:

```rust
use nsip::NsipClient;

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    let client = NsipClient::new();
    let ids = ["430735-0032", "430735-0041"];

    for id in &ids {
        let profile = client.search_by_lpn(id).await?;
        println!(
            "{}: breed={:?}, progeny_count={}",
            profile.details.lpn_id,
            profile.details.breed,
            profile.progeny.total_count,
        );
    }

    Ok(())
}
```

---

## Verify Results

1. Compare the number of results fetched against `total_count` to confirm completeness.
2. Check for errors in batch results -- individual animals may return `NotFound` without affecting others.
3. For large batches, monitor API response times and adjust `batch_size` if requests start timing out.

---

## See Also

- [How to Filter Search Results](FILTER-SEARCH-RESULTS.md) -- narrow down which animals to query
- [How to Export JSON](EXPORT-JSON.md) -- save batch results to files
- [How to Integrate with Scripts](SCRIPTING-INTEGRATION.md) -- automate batch queries in pipelines

# Error Handling Reference

Complete reference for error handling in the `nsip` crate.

---

## Error Type

````rust
pub enum Error {
    Validation(String),
    Api { status: u16, message: String },
    NotFound(String),
    Timeout(String),
    Connection(String),
    Parse(String),
}
````

---

## Error Variants

### `Error::Validation`

**When:** Invalid input parameters provided to an API method.

**Example causes:**
- Negative page size
- Invalid LPN ID format
- Null/empty required parameters

**Handling:**

````rust
match client.search_animals(0, 0, None, None, None, None).await {
    Err(Error::Validation(msg)) => {
        eprintln!("Invalid parameters: {}", msg);
        // Fix input and retry
    }
    Ok(results) => { /* ... */ }
    Err(e) => eprintln!("Other error: {}", e),
}
````

---

### `Error::Api`

**When:** The NSIP API returned a non-success HTTP status code.

**Fields:**
- `status: u16` - HTTP status code (400, 500, etc.)
- `message: String` - Human-readable error message

**Example causes:**
- 400 Bad Request - Malformed search criteria
- 500 Internal Server Error - Server-side issue
- 503 Service Unavailable - API temporarily down

**Handling:**

````rust
match client.breed_groups().await {
    Err(Error::Api { status, message }) => {
        match status {
            400 => eprintln!("Bad request: {}", message),
            500..=599 => {
                eprintln!("Server error: {}", message);
                // Retry logic here
            }
            _ => eprintln!("HTTP {}: {}", status, message),
        }
    }
    Ok(groups) => { /* ... */ }
    Err(e) => eprintln!("Other error: {}", e),
}
````

---

### `Error::NotFound`

**When:** The requested resource doesn't exist (HTTP 404).

**Example causes:**
- Animal with specified LPN ID not in database
- Invalid breed group ID

**Handling:**

````rust
match client.animal_details("INVALID_LPN").await {
    Err(Error::NotFound(msg)) => {
        eprintln!("Animal not found: {}", msg);
        // Prompt user for different ID
    }
    Ok(animal) => println!("Found: {}", animal.lpn_id),
    Err(e) => eprintln!("Other error: {}", e),
}
````

---

### `Error::Timeout`

**When:** Request exceeded the configured timeout duration.

**Example causes:**
- Slow network connection
- Large data transfer
- Server overload

**Handling:**

````rust
match client.search_animals(0, 1000, None, None, None, None).await {
    Err(Error::Timeout(msg)) => {
        eprintln!("Request timed out: {}", msg);
        // Increase timeout or reduce page size
        let client = NsipClient::builder().timeout_secs(120).build()?;
        // Retry with new client
    }
    Ok(results) => { /* ... */ }
    Err(e) => eprintln!("Other error: {}", e),
}
````

---

### `Error::Connection`

**When:** Failed to establish HTTP connection to the API.

**Example causes:**
- No internet connectivity
- DNS resolution failure
- Firewall blocking requests
- Invalid base URL

**Handling:**

````rust
match client.breed_groups().await {
    Err(Error::Connection(msg)) => {
        eprintln!("Connection failed: {}", msg);
        // Check network, retry with backoff
        tokio::time::sleep(Duration::from_secs(5)).await;
        // Retry
    }
    Ok(groups) => { /* ... */ }
    Err(e) => eprintln!("Other error: {}", e),
}
````

---

### `Error::Parse`

**When:** Failed to deserialize API response into expected data structures.

**Example causes:**
- API response format changed
- Corrupted data
- Unexpected null fields
- JSON syntax errors

**Handling:**

````rust
match client.trait_ranges(640).await {
    Err(Error::Parse(msg)) => {
        eprintln!("Failed to parse response: {}", msg);
        // Report bug, API may have changed
    }
    Ok(ranges) => { /* ... */ }
    Err(e) => eprintln!("Other error: {}", e),
}
````

---

## Result Type

The crate defines a type alias for convenience:

````rust
pub type Result<T> = std::result::Result<T, Error>;
````

**Usage:**

````rust
async fn fetch_animal(lpn_id: &str) -> nsip::Result<nsip::AnimalDetails> {
    let client = NsipClient::new();
    client.animal_details(lpn_id).await
}
````

---

## Retry Pattern

The `NsipClient` automatically retries on server errors (500-504) with exponential backoff.

**Default retry behavior:**
- Max retries: 3
- Backoff: 0.5 × 2^attempt seconds
- Retry on: 500, 502, 503, 504

**Customize retry policy:**

````rust
let client = NsipClient::builder()
    .max_retries(5)  // More aggressive
    .build()?;
````

**Disable retries:**

````rust
let client = NsipClient::builder()
    .max_retries(0)  // Fail fast
    .build()?;
````

---

## Error Display

All errors implement `std::fmt::Display`:

````rust
use nsip::Error;

let err = Error::NotFound("Animal ABC123 not found".to_string());
println!("{}", err);  // "not found: Animal ABC123 not found"
````

---

## Error Chaining

For application-level error handling, wrap `nsip::Error` in your own error type:

````rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("NSIP API error: {0}")]
    Nsip(#[from] nsip::Error),
    
    #[error("Database error: {0}")]
    Database(String),
}

async fn process_animal(lpn_id: &str) -> Result<(), AppError> {
    let client = NsipClient::new();
    let animal = client.animal_details(lpn_id).await?;  // Auto-converts via From
    // ... process animal
    Ok(())
}
````

---

## Best Practices

✅ **Match specific error variants** for targeted recovery  
✅ **Log errors with context** for debugging  
✅ **Use `?` operator** for error propagation  
✅ **Configure timeouts** based on use case  
✅ **Handle `NotFound` gracefully** in user-facing code  

❌ **Don't ignore errors** with `.unwrap()`  
❌ **Don't use generic error messages**  
❌ **Don't retry on `ValidationError`** - fix input instead  

---

## Examples

### Comprehensive Error Handling

````rust
use nsip::{NsipClient, Error};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let client = NsipClient::builder()
        .timeout_secs(60)
        .max_retries(3)
        .build()
        .expect("Failed to build client");

    let lpn_id = "ABC123";
    
    match fetch_with_retry(&client, lpn_id, 3).await {
        Ok(animal) => {
            println!("✓ Retrieved: {}", animal.lpn_id);
        }
        Err(e) => {
            eprintln!("✗ Failed after retries: {}", e);
            std::process::exit(1);
        }
    }
}

async fn fetch_with_retry(
    client: &NsipClient,
    lpn_id: &str,
    max_attempts: u32,
) -> nsip::Result<nsip::AnimalDetails> {
    let mut attempts = 0;
    
    loop {
        attempts += 1;
        
        match client.animal_details(lpn_id).await {
            Ok(animal) => return Ok(animal),
            Err(Error::NotFound(msg)) => {
                // Don't retry on not found
                return Err(Error::NotFound(msg));
            }
            Err(Error::Validation(msg)) => {
                // Don't retry on validation errors
                return Err(Error::Validation(msg));
            }
            Err(e) if attempts >= max_attempts => {
                // Exhausted retries
                return Err(e);
            }
            Err(e) => {
                eprintln!("Attempt {} failed: {}, retrying...", attempts, e);
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
            }
        }
    }
}
````

---

## See Also

- [How to Configure Timeout and Retries](../how-to/CONFIGURE-CLIENT.md)
- [Client Architecture](../explanation/CLIENT-ARCHITECTURE.md)
- [API Reference](../MCP.md)

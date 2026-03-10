---
diataxis_type: reference
---
# Error Handling Reference

Complete reference for error handling in the `nsip` crate.

---

## Error Type

The crate defines a single error enum with six variants, implemented using `thiserror`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("API error (HTTP {status}): {message}")]
    Api { status: u16, message: String },

    #[error("not found: {0}")]
    NotFound(String),

    #[error("request timed out: {0}")]
    Timeout(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("parse error: {0}")]
    Parse(String),
}
```

All variants implement `std::fmt::Display` and `std::error::Error`.

---

## Result Type Alias

The crate provides a convenience alias:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

Use it in your own functions to propagate `nsip` errors:

```rust
async fn fetch_animal(lpn_id: &str) -> nsip::Result<nsip::AnimalDetails> {
    let client = NsipClient::new();
    client.animal_details(lpn_id).await
}
```

---

## Error Variants

### `Error::Validation`

Returned when input parameters fail local validation before a request is sent to the API.

**Display format:** `validation error: {message}`

**Triggered by:**

| Method | Condition |
|--------|-----------|
| `trait_ranges(breed_id)` | `breed_id <= 0` |
| `search_animals(page, page_size, ...)` | `page_size == 0` or `page_size > 100` |
| `animal_details(search_string)` | `search_string` is empty or whitespace-only |
| `lineage(lpn_id)` | `lpn_id` is empty or whitespace-only |
| `progeny(lpn_id, page, page_size)` | `lpn_id` is empty, or `page_size == 0` |
| `search_by_lpn(lpn_id)` | `lpn_id` is empty or whitespace-only |
| `NsipClientBuilder::build()` | (not applicable -- see `Error::Connection`) |

**Example:**

```rust
use nsip::{NsipClient, Error};

let client = NsipClient::new();

// page_size of 0 triggers Validation
match client.search_animals(0, 0, None, None, None, None).await {
    Err(Error::Validation(msg)) => {
        eprintln!("Invalid input: {}", msg);
        // Fix the input -- do not retry
    }
    Ok(results) => { /* process results */ }
    Err(e) => eprintln!("Other error: {}", e),
}
```

**Recovery:** Fix the input parameters. Never retry on validation errors.

---

### `Error::Api`

Returned when the NSIP API responds with a non-success HTTP status code that is not 404 and not retryable (or retries are exhausted for 5xx codes).

**Display format:** `API error (HTTP {status}): {message}`

**Fields:**
- `status: u16` -- the HTTP status code
- `message: String` -- human-readable description

**Common status codes:**

| Status | Meaning |
|--------|---------|
| 400 | Bad request -- malformed search criteria |
| 403 | Forbidden -- access denied |
| 500 | Internal server error (after retries exhausted) |
| 502 | Bad gateway (after retries exhausted) |
| 503 | Service unavailable (after retries exhausted) |
| 504 | Gateway timeout (after retries exhausted) |

**Example:**

```rust
match client.breed_groups().await {
    Err(Error::Api { status, message }) => {
        match status {
            400 => eprintln!("Bad request: {}", message),
            500..=599 => eprintln!("Server error ({}): {}", status, message),
            _ => eprintln!("HTTP {}: {}", status, message),
        }
    }
    Ok(groups) => { /* process groups */ }
    Err(e) => eprintln!("Other error: {}", e),
}
```

**Recovery:** For 4xx errors, check your request parameters. For 5xx errors, the client has already retried according to its retry policy (see [Retry Behavior](#retry-behavior)). You may wait and retry later.

---

### `Error::NotFound`

Returned when the API responds with HTTP 404 -- the requested resource does not exist.

**Display format:** `not found: {message}`

**Triggered by:**
- `animal_details()` when the animal is not in the database
- `lineage()` when the LPN ID has no lineage data
- `progeny()` when the LPN ID has no progeny data
- Any endpoint that returns HTTP 404

**Example:**

```rust
match client.animal_details("NONEXISTENT-ID").await {
    Err(Error::NotFound(msg)) => {
        eprintln!("Not found: {}", msg);
        // Prompt user for a different ID
    }
    Ok(animal) => println!("Found: {}", animal.lpn_id),
    Err(e) => eprintln!("Other error: {}", e),
}
```

**Recovery:** Verify the LPN ID or search string is correct. Do not retry with the same identifier.

---

### `Error::Timeout`

Returned when the HTTP request exceeds the configured timeout duration. The default timeout is 30 seconds.

**Display format:** `request timed out: {message}`

**Triggered by:**
- Slow network connections
- Large result sets
- Server overload

**Example:**

```rust
match client.search_animals(0, 100, None, None, None, None).await {
    Err(Error::Timeout(msg)) => {
        eprintln!("Timed out: {}", msg);
        // Reduce page size or increase timeout
        let client = NsipClient::builder()
            .timeout_secs(120)
            .build()?;
    }
    Ok(results) => { /* process results */ }
    Err(e) => eprintln!("Other error: {}", e),
}
```

**Recovery:** Increase the timeout with `NsipClient::builder().timeout_secs()`, reduce the page size, or retry after a delay.

---

### `Error::Connection`

Returned when the HTTP client cannot establish a connection to the API server.

**Display format:** `connection error: {message}`

**Triggered by:**
- No internet connectivity
- DNS resolution failure
- Firewall blocking the request
- Invalid base URL configured via `NsipClient::with_base_url()` or `NsipClientBuilder::base_url()`
- Failure to build the `reqwest::Client` in `NsipClientBuilder::build()`

**Example:**

```rust
use std::time::Duration;

match client.breed_groups().await {
    Err(Error::Connection(msg)) => {
        eprintln!("Connection failed: {}", msg);
        // Check network, then retry
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    Ok(groups) => { /* process groups */ }
    Err(e) => eprintln!("Other error: {}", e),
}
```

**Recovery:** Check network connectivity and the configured base URL. Retry with exponential backoff.

---

### `Error::Parse`

Returned when the API response cannot be deserialized into the expected data type.

**Display format:** `parse error: {message}`

**Triggered by:**
- Unexpected JSON structure from the API
- Missing required fields in the response
- Invalid data types in the response
- API format changes

**Example:**

```rust
match client.trait_ranges(640).await {
    Err(Error::Parse(msg)) => {
        eprintln!("Parse error: {}", msg);
        // Likely an API change -- report as a bug
    }
    Ok(ranges) => { /* process ranges */ }
    Err(e) => eprintln!("Other error: {}", e),
}
```

**Recovery:** Parse errors typically indicate an API-side change. Report it as a bug. Do not retry with the same request.

---

## Retry Behavior

The `NsipClient` automatically retries requests that fail with specific server error codes. Retries happen transparently before any error is returned to the caller.

**Retried status codes:** 500, 502, 503, 504

**Default retry policy:**

| Setting | Default | Builder method |
|---------|---------|----------------|
| Max retries | 3 | `NsipClientBuilder::max_retries()` |
| Backoff factor | 0.5 | Not configurable |
| Backoff formula | `0.5 * 2^attempt` seconds | -- |

**Retry delay schedule (with defaults):**

| Attempt | Delay |
|---------|-------|
| 1 | 0.5 seconds |
| 2 | 1.0 seconds |
| 3 | 2.0 seconds |

If all retries are exhausted, the final error is returned as `Error::Api`.

**Customize retry policy:**

```rust
// More aggressive retries
let client = NsipClient::builder()
    .max_retries(5)
    .build()?;

// No retries (fail fast)
let client = NsipClient::builder()
    .max_retries(0)
    .build()?;
```

---

## Error Display Messages

Each variant produces a distinct display prefix:

| Variant | Display prefix |
|---------|---------------|
| `Validation(msg)` | `validation error: {msg}` |
| `Api { status, message }` | `API error (HTTP {status}): {message}` |
| `NotFound(msg)` | `not found: {msg}` |
| `Timeout(msg)` | `request timed out: {msg}` |
| `Connection(msg)` | `connection error: {msg}` |
| `Parse(msg)` | `parse error: {msg}` |

---

## Matching All Variants

A comprehensive match on all error variants:

```rust
use nsip::{NsipClient, Error};

let client = NsipClient::new();

match client.animal_details("430735-0032").await {
    Ok(animal) => {
        println!("Retrieved: {}", animal.lpn_id);
    }
    Err(Error::Validation(msg)) => {
        // Bad input -- fix and do not retry
        eprintln!("Invalid input: {}", msg);
    }
    Err(Error::Api { status, message }) => {
        // Server returned an error HTTP status
        eprintln!("API error (HTTP {}): {}", status, message);
    }
    Err(Error::NotFound(msg)) => {
        // Resource does not exist
        eprintln!("Not found: {}", msg);
    }
    Err(Error::Timeout(msg)) => {
        // Request exceeded timeout
        eprintln!("Timed out: {}", msg);
    }
    Err(Error::Connection(msg)) => {
        // Network-level failure
        eprintln!("Connection error: {}", msg);
    }
    Err(Error::Parse(msg)) => {
        // Response deserialization failed
        eprintln!("Parse error: {}", msg);
    }
}
```

---

## Wrapping in Application Errors

Use `#[from]` with `thiserror` to convert `nsip::Error` into your application's error type:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("NSIP error: {0}")]
    Nsip(#[from] nsip::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

async fn process_animal(lpn_id: &str) -> Result<(), AppError> {
    let client = nsip::NsipClient::new();
    let animal = client.animal_details(lpn_id).await?; // converts via From
    Ok(())
}
```

---

## See Also

- [Configuration Reference](CONFIGURATION.md) -- timeout and retry settings
- [Library API Reference](LIBRARY-API.md) -- method signatures and validation rules
- [How to Configure Timeout and Retries](../how-to/CONFIGURE-CLIENT.md)
- [NSIP Data Model](../explanation/NSIP-DATA-MODEL.md)

# How to Configure Timeout and Retries

> **Problem:** You need to adjust HTTP client timeouts or retry behavior for your specific use case.

**Time:** 5 minutes

---

## Default Configuration

The `NsipClient` uses sensible defaults:

- **Timeout:** 30 seconds per request
- **Max Retries:** 3 attempts on server errors (500, 502, 503, 504)
- **Backoff Strategy:** Exponential (0.5 × 2^attempt seconds)

````rust
use nsip::NsipClient;

// Uses defaults: 30s timeout, 3 retries
let client = NsipClient::new();
````

---

## Increase Timeout

For slow network connections or large data transfers:

````rust
use nsip::NsipClient;

let client = NsipClient::builder()
    .timeout_secs(120)  // 2 minutes
    .build()?;
````

---

## Disable Retries

For time-sensitive applications where you'd rather fail fast:

````rust
let client = NsipClient::builder()
    .max_retries(0)  // No retries
    .build()?;
````

---

## Aggressive Retry Policy

For unreliable networks where you want maximum resilience:

````rust
let client = NsipClient::builder()
    .timeout_secs(60)   // 1 minute per request
    .max_retries(10)     // Retry up to 10 times
    .build()?;
````

**Retry delays:** 0.5s, 1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s, 256s (total ~8.5 minutes)

---

## Custom Base URL

For testing or proxies:

````rust
let client = NsipClient::builder()
    .base_url("http://localhost:8080/api")
    .build()?;
````

---

## Combined Configuration

````rust
let client = NsipClient::builder()
    .base_url("http://nsipsearch.nsip.org/api")
    .timeout_secs(90)
    .max_retries(5)
    .build()?;

// Now use the client
let groups = client.breed_groups().await?;
````

---

## Error Handling

The builder returns `Result<NsipClient, Error>`:

````rust
use nsip::Error;

match NsipClient::builder().build() {
    Ok(client) => {
        // Use client
    }
    Err(Error::Validation(msg)) => {
        eprintln!("Invalid configuration: {}", msg);
    }
    Err(e) => {
        eprintln!("Failed to create client: {}", e);
    }
}
````

---

## See Also

- [Error Handling Reference](../reference/ERROR-HANDLING.md)
- [API Client Architecture](../explanation/CLIENT-ARCHITECTURE.md)

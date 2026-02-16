# How to Configure the NSIP Client

> **Problem:** You need to customize the HTTP client for timeouts, retries, or a different API endpoint.

**Prerequisites:**
- `nsip` crate added to your `Cargo.toml`
- A Tokio async runtime

---

## Step 1: Choose a Construction Method

The `NsipClient` offers three ways to create an instance, depending on how much control you need.

### Default Client

Use `NsipClient::new()` when the defaults are acceptable:

```rust
use nsip::NsipClient;

let client = NsipClient::new();
```

Defaults:
- **Base URL:** `http://nsipsearch.nsip.org/api`
- **Timeout:** 30 seconds per request
- **Max retries:** 3 (on status 500, 502, 503, 504)
- **Backoff:** Exponential (0.5 x 2^attempt seconds)

### Custom Base URL Only

Use `NsipClient::with_base_url()` when you only need to change the endpoint:

```rust
use nsip::NsipClient;

let client = NsipClient::with_base_url("http://localhost:8080/api");
```

### Full Builder

Use `NsipClient::builder()` when you need control over multiple settings:

```rust
use nsip::NsipClient;

let client = NsipClient::builder()
    .base_url("http://nsipsearch.nsip.org/api")
    .timeout_secs(60)
    .max_retries(5)
    .build()?;
```

---

## Step 2: Configure Timeout

The timeout controls how long each individual HTTP request waits before failing. The default is 30 seconds.

**Increase for slow networks or large responses:**

```rust
let client = NsipClient::builder()
    .timeout_secs(120)  // 2 minutes
    .build()?;
```

**Decrease for fast-fail scenarios:**

```rust
let client = NsipClient::builder()
    .timeout_secs(5)
    .build()?;
```

---

## Step 3: Configure Retry Behavior

The client automatically retries on server errors (HTTP 500, 502, 503, 504), timeouts, and connection failures. Client errors (4xx) are never retried.

**Default:** 3 retries with exponential backoff (0.5s, 1s, 2s, 4s, ...).

**Disable retries for time-sensitive applications:**

```rust
let client = NsipClient::builder()
    .max_retries(0)
    .build()?;
```

**Aggressive retries for unreliable networks:**

```rust
let client = NsipClient::builder()
    .timeout_secs(60)
    .max_retries(10)
    .build()?;
```

With 10 retries, backoff delays are: 0.5s, 1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s, 256s (total wait up to ~8.5 minutes before the final attempt).

---

## Step 4: Configure Base URL

Override the base URL for testing, proxies, or custom deployments:

```rust
let client = NsipClient::builder()
    .base_url("http://my-proxy.internal:3000/nsip-api")
    .build()?;
```

You can verify the configured URL at any time:

```rust
assert_eq!(client.base_url(), "http://my-proxy.internal:3000/nsip-api");
```

---

## Step 5: Handle Builder Errors

The `build()` method returns `Result<NsipClient>`. Handle failures explicitly:

```rust
use nsip::{NsipClient, Error};

match NsipClient::builder().timeout_secs(60).build() {
    Ok(client) => {
        let groups = client.breed_groups().await?;
    }
    Err(Error::Connection(msg)) => {
        eprintln!("Failed to create HTTP client: {msg}");
    }
    Err(e) => {
        eprintln!("Unexpected error: {e}");
    }
}
```

---

## Verify It Works

After building the client, make a lightweight call to verify connectivity:

```rust
let client = NsipClient::builder()
    .timeout_secs(10)
    .max_retries(1)
    .build()?;

let updated = client.date_last_updated().await?;
println!("Database last updated: {:?}", updated.data);
```

If this returns successfully, the client is configured correctly.

---

## Builder Options Reference

| Method            | Default                                | Description                                     |
|-------------------|----------------------------------------|-------------------------------------------------|
| `base_url(url)`   | `http://nsipsearch.nsip.org/api`       | API base URL                                    |
| `timeout_secs(n)` | 30                                     | Per-request timeout in seconds                  |
| `max_retries(n)`  | 3                                      | Max retry attempts on server/connection errors  |

---

## See Also

- [Error Handling Reference](../reference/ERROR-HANDLING.md)
- [Configuration Reference](../reference/CONFIGURATION.md)

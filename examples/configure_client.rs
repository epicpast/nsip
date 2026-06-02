//! Doc-example regression guard for `docs/how-to/CONFIGURE-CLIENT.md`.
//!
//! Combines the builder/retry configuration and the `build()` error-handling
//! `match` (Step 5) into one complete program so `cargo build --examples`
//! (via `just check-doc-examples`) catches regressions in the `builder()`,
//! `timeout_secs`, `max_retries`, and `Error` surfaces.
//!
//! Compile-checked only; never executed in CI (it issues live API calls).

// Doc examples are illustrative programs that print to stdout/stderr, mirroring
// what a user would write; the library's print bans do not apply here.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use nsip::{Error, NsipClient};

#[tokio::main]
async fn main() -> Result<(), nsip::Error> {
    // Full builder: timeout + retry configuration.
    let client = NsipClient::builder()
        .base_url("http://nsipsearch.nsip.org/api")
        .timeout_secs(60)
        .max_retries(5)
        .build()?;

    // Error handling around the builder (Step 5).
    match NsipClient::builder().timeout_secs(60).build() {
        Ok(client) => {
            let groups = client.breed_groups().await?;
            println!("Fetched {} breed groups", groups.len());
        },
        Err(Error::Connection { message, .. }) => {
            eprintln!("Failed to create HTTP client: {message}");
        },
        Err(e) => {
            eprintln!("Unexpected error: {e}");
        },
    }

    let updated = client.date_last_updated().await?;
    println!("Database last updated: {:?}", updated.data);

    Ok(())
}

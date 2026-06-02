//! Doc-example regression guard for `docs/tutorials/GETTING-STARTED.md` (Step 6).
//!
//! This is the canonical error-handling `match` from the tutorial, extracted
//! verbatim so `cargo build --examples` (via `just check-doc-examples`) fails
//! if the public `Error` surface or `animal_details` signature regresses.
//!
//! It is compile-checked only and never executed in CI: `main` issues a live
//! API call, so running it would hit the network.

// Doc examples are illustrative programs that print to stdout/stderr, mirroring
// what a user would write; the library's print bans do not apply here.
// `uninlined_format_args` is allowed so the example stays byte-for-byte faithful
// to the fenced block in GETTING-STARTED.md (which uses `"{}", var` form).
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::uninlined_format_args
)]

use nsip::{Error, NsipClient};

#[tokio::main]
async fn main() {
    let client = NsipClient::new();

    match client.animal_details("INVALID_ID").await {
        Ok(details) => {
            println!("Found: {}", details.lpn_id);
        },
        Err(Error::NotFound(msg)) => {
            eprintln!("Animal not found: {}", msg);
        },
        Err(Error::Timeout { message, .. }) => {
            eprintln!("Request timed out: {}", message);
        },
        Err(Error::Api {
            status, message, ..
        }) => {
            eprintln!("API error (HTTP {}): {}", status, message);
        },
        Err(Error::Connection { message, .. }) => {
            eprintln!("Connection failed: {}", message);
        },
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
        },
    }
}

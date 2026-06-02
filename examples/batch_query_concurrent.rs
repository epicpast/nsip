//! Doc-example regression guard for `docs/how-to/BATCH-QUERY.md` (Library
//! Method, Step 2 — concurrent batched fetch).
//!
//! Extracted verbatim so `cargo build --examples` (via
//! `just check-doc-examples`) fails if `AnimalDetails`, `animal_details`, or
//! the `futures::future::join_all` usage regresses. `futures` is a
//! dev-dependency used only by this example, never by the library.
//!
//! Compile-checked only; never executed in CI (it issues live API calls).

// Doc examples are illustrative programs that print to stdout/stderr, mirroring
// what a user would write; the library's print bans do not apply here.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use futures::future::join_all;
use nsip::{AnimalDetails, NsipClient};

async fn fetch_batch(
    client: &NsipClient,
    ids: &[&str],
    batch_size: usize,
) -> Vec<Result<AnimalDetails, nsip::Error>> {
    let mut results = Vec::new();

    for chunk in ids.chunks(batch_size) {
        // Build one future per ID, then drive them all concurrently.
        let futures = chunk.iter().map(|id| client.animal_details(id));
        results.extend(join_all(futures).await);
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

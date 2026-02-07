//! Binary entry point for NSIP Search API client.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nsip::{NsipClient, SearchCriteria};

/// NSIP Search API client CLI.
#[derive(Parser, Debug)]
#[command(name = "nsip")]
#[command(about = "NSIP Search API client for nsipsearch.nsip.org/api", long_about = None)]
#[command(version)]
struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Get the date when the database was last updated.
    DateUpdated,

    /// List all available breed groups.
    BreedGroups,

    /// List all available animal statuses.
    Statuses,

    /// Get trait ranges for a specific breed.
    TraitRanges {
        /// Breed ID to query trait ranges for.
        breed_id: i64,
    },

    /// Search for animals.
    Search {
        /// Breed ID to filter by.
        #[arg(short, long)]
        breed_id: Option<i64>,

        /// Animal status to filter by.
        #[arg(short, long)]
        status: Option<String>,

        /// Gender filter (Male, Female, Both).
        #[arg(short, long)]
        gender: Option<String>,

        /// Page number (0-indexed).
        #[arg(short, long, default_value = "0")]
        page: u32,

        /// Number of results per page (1-100).
        #[arg(long, default_value = "15")]
        page_size: u32,
    },

    /// Get detailed information about a specific animal.
    Details {
        /// LPN ID or registration number of the animal.
        search_string: String,
    },

    /// Get lineage (ancestry) information for a specific animal.
    Lineage {
        /// LPN ID of the animal.
        lpn_id: String,
    },

    /// Get progeny (offspring) information for a specific animal.
    Progeny {
        /// LPN ID of the animal.
        lpn_id: String,

        /// Page number (0-indexed).
        #[arg(short, long, default_value = "0")]
        page: u32,

        /// Number of results per page.
        #[arg(long, default_value = "10")]
        page_size: u32,
    },

    /// Get full profile (details + lineage + progeny) for an animal.
    Profile {
        /// LPN ID of the animal.
        lpn_id: String,
    },

    /// Start the MCP server for AI assistant integration.
    Mcp,
}

/// Runs the application logic.
#[allow(clippy::too_many_lines)]
async fn run() -> Result<(), nsip::Error> {
    let cli = Cli::parse();
    let client = NsipClient::new();

    match cli.command {
        Commands::DateUpdated => {
            let updated = client.date_last_updated().await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&updated.data).unwrap_or_default()
            );
        },

        Commands::BreedGroups => {
            let groups = client.breed_groups().await?;
            println!("Breed Groups:");
            for bg in groups {
                println!("  {} (ID: {})", bg.name, bg.id);
                for breed in &bg.breeds {
                    println!("    - {} (ID: {})", breed.name, breed.id);
                }
            }
        },

        Commands::Statuses => {
            let statuses = client.statuses().await?;
            println!("Animal Statuses:");
            for status in statuses {
                println!("  {status}");
            }
        },

        Commands::TraitRanges { breed_id } => {
            let ranges = client.trait_ranges(breed_id).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ranges).unwrap_or_default()
            );
        },

        Commands::Search {
            breed_id,
            status,
            gender,
            page,
            page_size,
        } => {
            let mut criteria = SearchCriteria::new();
            if let Some(bid) = breed_id {
                criteria = criteria.with_breed_id(bid);
            }
            if let Some(s) = status {
                criteria = criteria.with_status(s);
            }
            if let Some(g) = gender {
                criteria = criteria.with_gender(g);
            }

            let results = client
                .search_animals(page, page_size, breed_id, None, None, Some(&criteria))
                .await?;

            println!(
                "Search Results (page {}, page_size {}, total: {}):",
                results.page, results.page_size, results.total_count
            );

            for animal in &results.results {
                println!("  {}", serde_json::to_string(animal).unwrap_or_default());
            }
        },

        Commands::Details { search_string } => {
            let details = client.animal_details(&search_string).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&details).unwrap_or_default()
            );
        },

        Commands::Lineage { lpn_id } => {
            let lineage = client.lineage(&lpn_id).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&lineage).unwrap_or_default()
            );
        },

        Commands::Progeny {
            lpn_id,
            page,
            page_size,
        } => {
            let progeny = client.progeny(&lpn_id, page, page_size).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&progeny).unwrap_or_default()
            );
        },

        Commands::Profile { lpn_id } => {
            let profile = client.search_by_lpn(&lpn_id).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&profile).unwrap_or_default()
            );
        },

        Commands::Mcp => {
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .init();
            nsip::mcp::serve_stdio().await?;
        },
    }

    Ok(())
}

/// Main entry point.
#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        },
    }
}

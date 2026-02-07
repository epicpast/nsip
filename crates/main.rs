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
    /// List all available breed groups.
    BreedGroups,

    /// List all available animal statuses.
    Statuses,

    /// List available trait ranges.
    TraitRanges,

    /// Search for animals.
    Search {
        /// Breed group to filter by.
        #[arg(short, long)]
        breed_group: Option<String>,

        /// Animal status to filter by.
        #[arg(short, long)]
        status: Option<String>,

        /// Search query string.
        #[arg(short, long)]
        query: Option<String>,

        /// Page number for pagination.
        #[arg(short, long, default_value = "1")]
        page: u32,

        /// Number of results per page.
        #[arg(long, default_value = "20")]
        per_page: u32,
    },

    /// Get detailed information about a specific animal.
    Details {
        /// Unique identifier of the animal.
        animal_id: String,
    },

    /// Get lineage (ancestry) information for a specific animal.
    Lineage {
        /// Unique identifier of the animal.
        animal_id: String,
    },

    /// Get progeny (offspring) information for a specific animal.
    Progeny {
        /// Unique identifier of the animal.
        animal_id: String,
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
        Commands::BreedGroups => {
            let breed_groups = client.breed_groups().await?;
            println!("Breed Groups:");
            for bg in breed_groups {
                println!("  {} ({})", bg.name, bg.id);
                if let Some(desc) = &bg.description {
                    println!("    {desc}");
                }
            }
        },

        Commands::Statuses => {
            let statuses = client.statuses().await?;
            println!("Animal Statuses:");
            for status in statuses {
                println!("  {} ({})", status.name, status.id);
            }
        },

        Commands::TraitRanges => {
            let trait_ranges = client.trait_ranges().await?;
            println!("Trait Ranges:");
            for tr in trait_ranges {
                let unit = tr.unit.as_deref().unwrap_or("");
                println!(
                    "  {}: {}{} - {}{}",
                    tr.trait_name, tr.min_value, unit, tr.max_value, unit
                );
            }
        },

        Commands::Search {
            breed_group,
            status,
            query,
            page,
            per_page,
        } => {
            let mut criteria = SearchCriteria::new()
                .with_page(page)
                .with_per_page(per_page);

            if let Some(bg) = breed_group {
                criteria = criteria.with_breed_group(bg);
            }
            if let Some(s) = status {
                criteria = criteria.with_status(s);
            }
            if let Some(q) = query {
                criteria = criteria.with_query(q);
            }

            let results = client.search_animals(&criteria).await?;
            println!(
                "Search Results (page {}/{}, total: {}):",
                results.page,
                results.total.div_ceil(results.per_page as usize),
                results.total
            );

            for animal in results.animals {
                println!("\n  {} ({})", animal.name, animal.id);
                if let Some(breed) = animal.breed {
                    println!("    Breed: {breed}");
                }
                if let Some(status) = animal.status {
                    println!("    Status: {status}");
                }
                if let Some(birth_date) = animal.birth_date {
                    println!("    Birth Date: {birth_date}");
                }
            }
        },

        Commands::Details { animal_id } => {
            let animal = client.details(&animal_id).await?;
            println!("Animal Details:");
            println!("  ID: {}", animal.id);
            println!("  Name: {}", animal.name);

            if let Some(breed) = animal.breed {
                println!("  Breed: {breed}");
            }
            if let Some(breed_group) = animal.breed_group {
                println!("  Breed Group: {breed_group}");
            }
            if let Some(status) = animal.status {
                println!("  Status: {status}");
            }
            if let Some(birth_date) = animal.birth_date {
                println!("  Birth Date: {birth_date}");
            }
            if let Some(sex) = animal.sex {
                println!("  Sex: {sex}");
            }
            if let Some(sire) = animal.sire {
                println!("  Sire: {sire}");
            }
            if let Some(dam) = animal.dam {
                println!("  Dam: {dam}");
            }
        },

        Commands::Lineage { animal_id } => {
            let lineage = client.lineage(&animal_id).await?;
            println!("Lineage for Animal ID: {}", lineage.animal_id);
            println!("\nAncestors:");

            for ancestor in lineage.ancestors {
                println!("  {} ({})", ancestor.name, ancestor.id);
                if let Some(breed) = ancestor.breed {
                    println!("    Breed: {breed}");
                }
            }
        },

        Commands::Progeny { animal_id } => {
            let progeny = client.progeny(&animal_id).await?;
            println!("Progeny for Animal ID: {}", progeny.animal_id);
            println!("\nOffspring:");

            for offspring in progeny.offspring {
                println!("  {} ({})", offspring.name, offspring.id);
                if let Some(breed) = offspring.breed {
                    println!("    Breed: {breed}");
                }
            }
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

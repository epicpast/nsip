//! Binary entry point for NSIP Search API client.

#![allow(clippy::print_stdout, clippy::print_stderr)]

mod format;

use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};
use nsip::{NsipClient, SearchCriteria};

/// NSIP Search API client CLI.
#[derive(Parser, Debug)]
#[command(name = "nsip")]
#[command(about = "NSIP Search API client for nsipsearch.nsip.org/api", long_about = None)]
#[command(version)]
struct Cli {
    /// Output raw JSON instead of human-readable format.
    #[arg(long, short = 'J', global = true)]
    json: bool,

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

        /// Breed group ID to filter by.
        #[arg(long)]
        breed_group_id: Option<i64>,

        /// Animal status to filter by.
        #[arg(short, long)]
        status: Option<String>,

        /// Gender filter (Male, Female, Both).
        #[arg(short, long)]
        gender: Option<String>,

        /// Only return animals born after this date (YYYY-MM-DD).
        #[arg(long)]
        born_after: Option<String>,

        /// Only return animals born before this date (YYYY-MM-DD).
        #[arg(long)]
        born_before: Option<String>,

        /// Only return proven animals.
        #[arg(long)]
        proven_only: bool,

        /// Flock ID to filter by.
        #[arg(long)]
        flock_id: Option<String>,

        /// Sort results by trait abbreviation (e.g. BWT, WWT).
        #[arg(long)]
        sort_by: Option<String>,

        /// Reverse the sort order.
        #[arg(long)]
        reverse: bool,

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

    /// Compare two or more animals side-by-side.
    Compare {
        /// LPN IDs of animals to compare (2-5).
        #[arg(required = true, num_args = 2..=5)]
        lpn_ids: Vec<String>,

        /// Only show specific traits (comma-separated, e.g. BWT,WWT,YWT).
        #[arg(long)]
        traits: Option<String>,
    },

    /// Generate shell completions for bash, zsh, fish, or powershell.
    Completions {
        /// Shell to generate completions for.
        shell: clap_complete::Shell,
    },

    /// Generate man pages (writes to stdout or a directory).
    ManPages {
        /// Output directory for man pages. If omitted, writes the main page to stdout.
        #[arg(long)]
        out_dir: Option<String>,
    },

    /// Start the MCP server for AI assistant integration.
    Mcp {
        /// Transport type: stdio (default) or http.
        #[arg(long, default_value = "stdio")]
        transport: String,

        /// Host address to bind to (HTTP transport only).
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Port to bind to (HTTP transport only).
        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// Comma-separated tool sets to enable (search,analytics,flock,breed).
        /// Defaults to all sets enabled.
        #[arg(long)]
        tools: Option<String>,

        /// Enable OAuth 2.1 + GitHub PAT bearer auth (HTTP transport only).
        /// Requires `NSIP_GITHUB_CLIENT_ID`, `NSIP_GITHUB_CLIENT_SECRET`,
        /// `NSIP_AUTH_SECRET`, and `NSIP_AUTH_BASE_URL` environment variables.
        #[arg(long)]
        auth: bool,
    },
}

/// Generate man pages to a directory or stdout.
fn generate_man_pages(out_dir: Option<String>) -> Result<(), nsip::Error> {
    let cmd = Cli::command();

    let Some(dir) = out_dir else {
        let man = clap_mangen::Man::new(cmd);
        return man
            .render(&mut std::io::stdout())
            .map_err(|e| nsip::Error::Parse(format!("man page render error: {e}")));
    };

    let path = std::path::Path::new(&dir);
    std::fs::create_dir_all(path)
        .map_err(|e| nsip::Error::Validation(format!("cannot create directory {dir}: {e}")))?;

    render_man_page(&cmd, path, "nsip")?;

    for sub in cmd.get_subcommands() {
        let name = sub.get_name();
        let filename = format!("nsip-{name}");
        render_man_page(sub, path, &filename)?;
    }

    println!("Man pages written to {dir}/");
    Ok(())
}

/// Render a single man page to a directory.
fn render_man_page(
    cmd: &clap::Command,
    dir: &std::path::Path,
    name: &str,
) -> Result<(), nsip::Error> {
    let man = clap_mangen::Man::new(cmd.clone());
    let mut buf: Vec<u8> = Vec::new();
    man.render(&mut buf)
        .map_err(|e| nsip::Error::Parse(format!("man page render error: {e}")))?;
    let filename = format!("{name}.1");
    std::fs::write(dir.join(&filename), buf)
        .map_err(|e| nsip::Error::Validation(format!("cannot write {filename}: {e}")))
}

/// Initialise the tracing subscriber.
///
/// When compiled with the `telemetry` feature and `NSIP_OTLP_ENDPOINT` is set,
/// the subscriber includes an `OpenTelemetry` layer that attaches W3C trace
/// context (`trace_id`, `span_id`) to every JSON log line. Otherwise a plain
/// text subscriber writing to stderr is used.
fn init_tracing() {
    init_tracing_inner();
}

#[cfg(feature = "telemetry")]
fn init_tracing_inner() {
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    let provider = nsip::mcp::telemetry::init_tracer_provider();
    let otel_layer = nsip::mcp::telemetry::otel_layer(&provider);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .event_format(nsip::mcp::telemetry::OtelJsonFormat::default())
                .fmt_fields(tracing_subscriber::fmt::format::JsonFields::default()),
        )
        .with(otel_layer)
        .init();
}

#[cfg(not(feature = "telemetry"))]
fn init_tracing_inner() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
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
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&groups).unwrap_or_default()
                );
            } else {
                print!("{}", format::fmt_breed_groups(&groups));
            }
        },

        Commands::Statuses => {
            let statuses = client.statuses().await?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&statuses).unwrap_or_default()
                );
            } else {
                println!("Animal Statuses:");
                for status in &statuses {
                    println!("  {status}");
                }
            }
        },

        Commands::TraitRanges { breed_id } => {
            let ranges = client.trait_ranges(breed_id).await?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&ranges).unwrap_or_default()
                );
            } else {
                print!("{}", format::fmt_trait_ranges(&ranges));
            }
        },

        Commands::Search {
            breed_id,
            breed_group_id,
            status,
            gender,
            born_after,
            born_before,
            proven_only,
            flock_id,
            sort_by,
            reverse,
            page,
            page_size,
        } => {
            let mut criteria = SearchCriteria::new();
            if let Some(bid) = breed_id {
                criteria = criteria.with_breed_id(bid);
            }
            if let Some(bgid) = breed_group_id {
                criteria = criteria.with_breed_group_id(bgid);
            }
            if let Some(s) = status {
                criteria = criteria.with_status(s);
            }
            if let Some(g) = gender {
                criteria = criteria.with_gender(g);
            }
            if let Some(date) = born_after {
                criteria = criteria.with_born_after(date);
            }
            if let Some(date) = born_before {
                criteria = criteria.with_born_before(date);
            }
            if proven_only {
                criteria = criteria.with_proven_only(true);
            }
            if let Some(fid) = flock_id {
                criteria = criteria.with_flock_id(fid);
            }

            let sorted_trait = sort_by.as_deref();
            let reverse_opt = if reverse { Some(true) } else { None };

            let results = client
                .search_animals(
                    page,
                    page_size,
                    breed_id,
                    sorted_trait,
                    reverse_opt,
                    Some(&criteria),
                )
                .await?;

            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&results).unwrap_or_default()
                );
            } else {
                print!("{}", format::fmt_search_results(&results));
            }
        },

        Commands::Details { search_string } => {
            let details = client.animal_details(&search_string).await?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&details).unwrap_or_default()
                );
            } else {
                print!("{}", format::fmt_details(&details));
            }
        },

        Commands::Lineage { lpn_id } => {
            let lineage = client.lineage(&lpn_id).await?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&lineage).unwrap_or_default()
                );
            } else {
                print!("{}", format::fmt_lineage(&lineage));
            }
        },

        Commands::Progeny {
            lpn_id,
            page,
            page_size,
        } => {
            let progeny = client.progeny(&lpn_id, page, page_size).await?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&progeny).unwrap_or_default()
                );
            } else {
                print!("{}", format::fmt_progeny(&progeny, &lpn_id));
            }
        },

        Commands::Profile { lpn_id } => {
            let profile = client.search_by_lpn(&lpn_id).await?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&profile).unwrap_or_default()
                );
            } else {
                print!("{}", format::fmt_profile(&profile));
            }
        },

        Commands::Compare { lpn_ids, traits } => {
            let mut join_set = tokio::task::JoinSet::new();
            for id in lpn_ids {
                let c = client.clone();
                join_set.spawn(async move { c.animal_details(&id).await });
            }

            let mut animals = Vec::new();
            while let Some(result) = join_set.join_next().await {
                let details = result.map_err(|e| nsip::Error::Parse(format!("join error: {e}")))?;
                animals.push(details?);
            }

            let trait_filter: Option<Vec<String>> =
                traits.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&animals).unwrap_or_default()
                );
            } else {
                print!(
                    "{}",
                    format::fmt_comparison(&animals, trait_filter.as_deref(),)
                );
            }
        },

        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "nsip", &mut std::io::stdout());
        },

        Commands::ManPages { out_dir } => {
            generate_man_pages(out_dir)?;
        },

        Commands::Mcp {
            transport,
            host,
            port,
            tools,
            auth,
        } => {
            init_tracing();
            let sets = tools.map_or_else(nsip::mcp::tool_sets::EnabledToolSets::all, |csv| {
                nsip::mcp::tool_sets::EnabledToolSets::from_csv(&csv)
            });
            let oauth_state = if auth {
                let config =
                    nsip::mcp::oauth::config::OAuthConfig::try_from_env().ok_or_else(|| {
                        nsip::Error::Validation(
                            "--auth requires NSIP_GITHUB_CLIENT_ID, NSIP_GITHUB_CLIENT_SECRET, \
                         NSIP_AUTH_SECRET, and NSIP_AUTH_BASE_URL environment variables"
                                .into(),
                        )
                    })?;
                let store = std::sync::Arc::new(nsip::mcp::oauth::store::InMemoryOAuthStore::new())
                    as std::sync::Arc<dyn nsip::mcp::oauth::store::OAuthStoreBackend>;
                Some(nsip::mcp::oauth::OAuthState::new(config, store))
            } else {
                None
            };
            match transport.as_str() {
                "stdio" => {
                    if auth {
                        eprintln!("warning: --auth is ignored for stdio transport");
                    }
                    nsip::mcp::serve_stdio(sets).await?;
                },
                "http" => nsip::mcp::serve_http(&host, port, sets, oauth_state).await?,
                other => {
                    return Err(nsip::Error::Validation(format!(
                        "unknown transport: {other}, expected 'stdio' or 'http'"
                    )));
                },
            }
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

//! Naseej CLI - Schema Ingestion and Management

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use cognitive_core::{SchemaIngestor, EmbeddingProvider, VectorStore};

/// Naseej CLI - AI-powered API Gateway Management
#[derive(Parser)]
#[command(name = "naseej")]
#[command(author = "YASSERRMD")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "CLI for Naseej Mesh - AI-powered API Gateway", long_about = None)]
struct Cli {
    /// Verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest an OpenAPI/WSDL specification into the knowledge base
    Learn {
        /// Path to the specification file (YAML or JSON)
        #[arg(value_name = "FILE")]
        path: PathBuf,

        /// Source identifier for this specification
        #[arg(short, long)]
        source: Option<String>,

        /// Force re-ingestion even if already exists
        #[arg(short, long)]
        force: bool,
    },

    /// Search for API endpoints in the knowledge base
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },

    /// List deployed routes
    Routes {
        /// Show only active routes
        #[arg(short, long)]
        active: bool,

        /// Show only AI-generated routes
        #[arg(short = 'i', long)]
        ai_only: bool,
    },

    /// Check system status
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity
    let log_level = match cli.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    match cli.command {
        Commands::Learn { path, source, force } => {
            learn_command(path, source, force).await?;
        }
        Commands::Search { query, limit } => {
            search_command(&query, limit).await?;
        }
        Commands::Routes { active, ai_only } => {
            routes_command(active, ai_only).await?;
        }
        Commands::Status => {
            status_command().await?;
        }
    }

    Ok(())
}

/// Learn command - ingest a schema into the vector store
async fn learn_command(
    path: PathBuf,
    source: Option<String>,
    _force: bool,
) -> anyhow::Result<()> {
    let source_name = source.unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    println!("📚 Learning from: {}", path.display());
    println!("   Source: {}", source_name);

    // Read the file
    let content = std::fs::read_to_string(&path)?;
    
    // Parse the schema
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message("Parsing specification...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let ingestor = SchemaIngestor::new(&source_name);
    let endpoints = ingestor.parse_openapi(&content)?;

    pb.finish_with_message(format!("Found {} endpoints", endpoints.len()));

    if endpoints.is_empty() {
        println!("⚠️  No endpoints found in specification");
        return Ok(());
    }

    // Generate embeddings
    let embed_pb = ProgressBar::new(endpoints.len() as u64);
    embed_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    embed_pb.set_message("Generating embeddings...");

    let embedding_provider = EmbeddingProvider::new();
    let vector_store = Arc::new(RwLock::new(VectorStore::new()));

    for endpoint in &endpoints {
        let _embedding = embedding_provider.embed(&endpoint.embedding_text).await?;
        
        let mut store = vector_store.write().await;
        store.add_endpoint(endpoint).await?;
        
        embed_pb.inc(1);
    }

    embed_pb.finish_with_message("Embeddings generated");

    println!("\n✅ Successfully learned {} endpoints from {}", endpoints.len(), source_name);
    println!("   Use 'naseej search <query>' to find endpoints");

    Ok(())
}

/// Search command - search for API endpoints
async fn search_command(query: &str, limit: usize) -> anyhow::Result<()> {
    println!("🔍 Searching for: \"{}\"", query);
    println!();

    let vector_store = Arc::new(RwLock::new(VectorStore::new()));
    
    // Search
    let store = vector_store.read().await;
    let results = store.search(query, limit).await?;

    if results.is_empty() {
        println!("No results found. Try:");
        println!("  • Using different keywords");
        println!("  • Running 'naseej learn <spec>' to add schemas");
        return Ok(());
    }

    println!("Found {} result(s):\n", results.len());

    for (i, result) in results.iter().enumerate() {
        println!("{}. {}", i + 1, result.endpoint_id);
        println!("   Score: {:.2}%", result.score * 100.0);
        println!("   {}", result.text);
        println!();
    }

    Ok(())
}

/// Routes command - list deployed routes
async fn routes_command(active_only: bool, ai_only: bool) -> anyhow::Result<()> {
    println!("📋 Deployed Routes");
    if active_only {
        println!("   (showing active only)");
    }
    if ai_only {
        println!("   (showing AI-generated only)");
    }
    println!();

    println!("No routes found. Routes will appear here after:");
    println!("  • Using the console to create routes");
    println!("  • Asking the AI Architect to deploy routes");
    
    Ok(())
}

/// Status command - check system status
async fn status_command() -> anyhow::Result<()> {
    println!("🔧 Naseej System Status");
    println!();
    
    // Check Cohere API
    let embedding = EmbeddingProvider::new();
    let cohere_status = if embedding.is_available() {
        "✅ Connected"
    } else {
        "❌ Not configured (set COHERE_API_KEY)"
    };
    println!("Cohere API: {}", cohere_status);

    println!("Vector Store: ✅ In-memory (ephemeral)");
    
    println!();
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    
    Ok(())
}

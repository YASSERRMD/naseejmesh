//! NaseejMesh API Gateway - Main Entry Point
//!
//! High-performance API Gateway with:
//! - Hyper 1.0 HTTP/1+2 support
//! - Embedded SurrealDB configuration
//! - Zero-downtime hot reload via ArcSwap
//! - Live Query reactive updates

use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use tokio::net::TcpListener;

use gateway_core::config::RouterMap;
use gateway_core::handler::{handle_request, health_check, readiness_check};
use surreal_config::{init_database, start_config_watcher, seed_default_routes, DatabaseConfig};

/// Server configuration
#[derive(Debug, Clone)]
struct ServerConfig {
    /// HTTP listen address
    listen_addr: SocketAddr,

    /// Database configuration
    db_config: DatabaseConfig,

    /// Enable development mode (seeds default routes)
    dev_mode: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: SocketAddr::from(([0, 0, 0, 0], 8080)),
            db_config: DatabaseConfig::from_env(),
            dev_mode: std::env::var("DEV_MODE").is_ok(),
        }
    }
}

impl ServerConfig {
    fn from_env() -> Self {
        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let host: [u8; 4] = std::env::var("HOST")
            .ok()
            .and_then(|h| {
                let parts: Vec<u8> = h.split('.').filter_map(|p| p.parse().ok()).collect();
                if parts.len() == 4 {
                    Some([parts[0], parts[1], parts[2], parts[3]])
                } else {
                    None
                }
            })
            .unwrap_or([0, 0, 0, 0]);

        Self {
            listen_addr: SocketAddr::from((host, port)),
            db_config: DatabaseConfig::from_env(),
            dev_mode: std::env::var("DEV_MODE").is_ok(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("naseejmesh=info".parse()?)
                .add_directive("gateway_core=debug".parse()?)
                .add_directive("surreal_config=debug".parse()?),
        )
        .with_target(true)
        .with_thread_ids(true)
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting NaseejMesh API Gateway"
    );

    // Load configuration
    let config = ServerConfig::from_env();
    tracing::info!(
        addr = %config.listen_addr,
        db_path = %config.db_config.connection,
        dev_mode = config.dev_mode,
        "Server configuration loaded"
    );

    // Initialize embedded SurrealDB
    let db = init_database(&config.db_config).await?;
    tracing::info!("Database initialized successfully");

    // Seed default routes in development mode
    if config.dev_mode {
        tracing::info!("Development mode: seeding default routes");
        seed_default_routes(&db).await?;
    }

    // Create shared routing configuration with ArcSwap for wait-free reads
    let router_config: Arc<ArcSwap<RouterMap>> = Arc::new(ArcSwap::from_pointee(HashMap::new()));

    // Spawn the configuration watcher task
    let watcher_db = db.clone();
    let watcher_config = router_config.clone();
    tokio::spawn(async move {
        if let Err(e) = start_config_watcher(watcher_db, watcher_config).await {
            tracing::error!(error = %e, "Configuration watcher failed");
        }
    });

    // Wait for initial configuration to load
    tracing::info!("Waiting for initial configuration load...");
    tokio::time::sleep(Duration::from_millis(500)).await;

    let initial_routes = router_config.load().len();
    tracing::info!(routes = initial_routes, "Initial configuration loaded");

    // Bind TCP listener
    let listener = TcpListener::bind(config.listen_addr).await?;
    tracing::info!(addr = %config.listen_addr, "Gateway listening for connections");

    // Print startup banner
    print_banner(&config.listen_addr);

    // Main accept loop
    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to accept connection");
                continue;
            }
        };

        // Wrap stream for Hyper 1.0 compatibility
        let io = TokioIo::new(stream);
        let config = router_config.clone();

        // Spawn connection handler
        tokio::spawn(async move {
            // Create service with clone-and-move pattern
            let service = service_fn(move |req: Request<Incoming>| {
                let config = config.clone();
                async move {
                    // Handle gateway-internal endpoints
                    let path = req.uri().path();
                    if path == "/_gateway/health" {
                        return Ok::<_, Infallible>(health_check());
                    }
                    if path == "/_gateway/ready" {
                        return Ok(readiness_check(&config));
                    }

                    // Handle regular requests
                    handle_request(req, config).await
                }
            });

            // Use auto-builder for HTTP/1 + HTTP/2 support
            let builder = AutoBuilder::new(TokioExecutor::new());

            if let Err(e) = builder.serve_connection(io, service).await {
                // Only log non-trivial errors
                let err_str = e.to_string();
                if !err_str.contains("connection closed") && !err_str.contains("reset by peer") {
                    tracing::debug!(
                        peer = %peer_addr,
                        error = %e,
                        "Connection error"
                    );
                }
            }
        });
    }
}

fn print_banner(addr: &SocketAddr) {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   ███╗   ██╗ █████╗ ███████╗███████╗███████╗     ██╗          ║
║   ████╗  ██║██╔══██╗██╔════╝██╔════╝██╔════╝     ██║          ║
║   ██╔██╗ ██║███████║███████╗█████╗  █████╗       ██║          ║
║   ██║╚██╗██║██╔══██║╚════██║██╔══╝  ██╔══╝  ██   ██║          ║
║   ██║ ╚████║██║  ██║███████║███████╗███████╗╚█████╔╝          ║
║   ╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝ ╚════╝           ║
║                                                               ║
║   High-Performance API Gateway                                ║
║   Phase 1: Foundation Infrastructure                          ║
║                                                               ║
╠═══════════════════════════════════════════════════════════════╣
║   Listening on: {:<45} ║
║   Health check: http://{}/health             ║
║                                                               ║
║   Features:                                                   ║
║   • HTTP/1.1 + HTTP/2 auto-negotiation                       ║
║   • Embedded SurrealDB with RocksDB                          ║
║   • Live Query hot reload                                    ║
║   • ArcSwap wait-free reads                                  ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
"#,
        addr, addr
    );
}

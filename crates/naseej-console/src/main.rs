//! # Naseej Console API Server
//!
//! Provides the backend API for the Visual Control Plane dashboard.
//! Features:
//! - Route management endpoints
//! - Transformation simulation (dry-run)
//! - AI chat via SSE streaming
//! - Real-time gateway state

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod handlers;
mod state;

pub use state::AppState;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3001,
        }
    }
}

/// Start the console API server
pub async fn run_server(config: ServerConfig) -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    // Create application state
    let state = AppState::new();

    // Build router
    let app = create_router(state);

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "Starting Naseej Console API server");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the Axum router
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Status & Metrics (NEW)
        .route("/api/status", get(handlers::get_status))
        .route("/api/metrics", get(handlers::get_metrics))
        // Routes management
        .route("/api/routes", get(handlers::list_routes))
        .route("/api/routes", post(handlers::create_route))
        // Transformations (NEW)
        .route("/api/transformations", get(handlers::list_transformations))
        // Security (NEW)
        .route("/api/security/events", get(handlers::list_security_events))
        // Schemas (NEW)
        .route("/api/schemas", get(handlers::list_schemas))
        // Transformation simulation
        .route("/api/simulate", post(handlers::simulate_transform))
        .route("/api/validate", post(handlers::validate_transform))
        // AI Chat
        .route("/api/chat", post(handlers::chat))
        .route("/api/chat/stream", get(handlers::chat_stream))
        // Gateway state
        .route("/api/state", get(handlers::get_state))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(Arc::new(state))
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::default();
    run_server(config).await
}

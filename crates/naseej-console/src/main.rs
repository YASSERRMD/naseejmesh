//! # Naseej Console API Server
//!
//! Provides the backend API for the Visual Control Plane dashboard.
//! Features:
//! - Route management endpoints
//! - Transformation simulation (dry-run)
//! - AI chat via SSE streaming
//! - AI chat via SSE streaming
//! - Real-time gateway state
//! - Persistent configuration via SurrealDB

use surreal_config::{DatabaseConfig, init_remote_database};

use axum::{
    routing::{get, post, delete},
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

    // Initialize Database
    let db_config = DatabaseConfig::from_env();
    info!("Initializing database connection...");
    let db = init_remote_database(&db_config).await?;

    // Seed default admin user
    seed_admin_user(&db).await?;

    let db = Arc::new(db);

    // Create application state
    let state = AppState::new(db);

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
        // Auth Routes
        .route("/api/auth/login", post(handlers::auth::login))
        // Admin Routes - Users
        .route("/api/admin/users", get(handlers::admin::list_users_handler))
        .route("/api/admin/users", post(handlers::admin::create_user_handler))
        // Admin Routes - Roles
        .route("/api/admin/roles", get(handlers::admin::list_roles_handler))
        .route("/api/admin/roles", post(handlers::admin::create_role_handler))
        // Admin Routes - API Keys
        .route("/api/admin/keys", get(handlers::admin::list_keys_handler))
        .route("/api/admin/keys", post(handlers::admin::create_key_handler))
        .route("/api/admin/keys/:id", delete(handlers::admin::delete_key_handler))
        // Schemas (NEW)
        .route("/api/schemas", get(handlers::list_schemas))
        // Transformation simulation
        .route("/api/simulate", post(handlers::simulate_transform))
        .route("/api/validate", post(handlers::validate_transform))
        // AI Chat
        .route("/api/chat", post(handlers::chat))
        .route("/api/chat/stream", get(handlers::chat_stream))
        // AI Design (Smart Design)
        .route("/api/design/generate", post(handlers::design::generate_flow))
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

/// Seed default admin user if no users exist
async fn seed_admin_user(db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>) -> anyhow::Result<()> {
    use surreal_config::auth_schema::{list_users, create_user};
    use gateway_core::auth::User;
    use naseej_security::KeyManager;
    use uuid::Uuid;

    let users = list_users(db).await?;
    if users.is_empty() {
        info!("Seeding default admin user...");
        let password_hash = KeyManager::hash_password("admin123")
            .map_err(|e| anyhow::anyhow!("Hashing failed: {}", e))?;

        let user = User {
            id: Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            password_hash,
            roles: vec!["admin".to_string()],
            active: true,
            created_at: chrono::Utc::now(),
        };

        create_user(db, user).await?;
        info!("Default admin user created (admin/admin123)");
    }

    Ok(())
}

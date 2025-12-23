//! Database initialization and connection management.
//!
//! Supports both embedded RocksDB (production) and remote SurrealDB (testing).

use surrealdb::engine::local::RocksDb;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use crate::error::ConfigError;

/// Database connection type alias for embedded RocksDB
pub type EmbeddedDb = Surreal<surrealdb::engine::local::Db>;

/// Database connection type alias for remote WebSocket connection
pub type RemoteDb = Surreal<Client>;

/// Configuration for database initialization
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path for embedded database or URL for remote
    pub connection: String,

    /// Namespace to use
    pub namespace: String,

    /// Database name within the namespace
    pub database: String,

    /// Use embedded RocksDB (true) or remote WebSocket (false)
    pub embedded: bool,

    /// Username for remote connection (ignored for embedded)
    pub username: Option<String>,

    /// Password for remote connection (ignored for embedded)
    pub password: Option<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection: "./data/gateway.db".to_string(),
            namespace: "gateway".to_string(),
            database: "config".to_string(),
            embedded: true,
            username: None,
            password: None,
        }
    }
}

impl DatabaseConfig {
    /// Create config for embedded RocksDB
    pub fn embedded(path: impl Into<String>) -> Self {
        Self {
            connection: path.into(),
            embedded: true,
            ..Default::default()
        }
    }

    /// Create config for remote SurrealDB (for testing with Docker)
    pub fn remote(url: impl Into<String>, username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            connection: url.into(),
            embedded: false,
            username: Some(username.into()),
            password: Some(password.into()),
            ..Default::default()
        }
    }

    /// Create config from environment variables
    pub fn from_env() -> Self {
        let embedded = std::env::var("SURREAL_EMBEDDED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        if embedded {
            Self::embedded(
                std::env::var("SURREAL_PATH")
                    .unwrap_or_else(|_| "./data/gateway.db".to_string())
            )
        } else {
            Self::remote(
                std::env::var("SURREAL_URL")
                    .unwrap_or_else(|_| "ws://localhost:8000".to_string()),
                std::env::var("SURREAL_USER")
                    .unwrap_or_else(|_| "root".to_string()),
                std::env::var("SURREAL_PASS")
                    .unwrap_or_else(|_| "root".to_string()),
            )
        }
    }
}

/// Initialize an embedded SurrealDB instance with RocksDB backend.
///
/// This creates a self-contained database that persists to the specified path.
/// The database is created if it doesn't exist, or recovered if it does.
pub async fn init_database(config: &DatabaseConfig) -> Result<EmbeddedDb, ConfigError> {
    tracing::info!(
        path = %config.connection,
        namespace = %config.namespace,
        database = %config.database,
        "Initializing embedded SurrealDB with RocksDB"
    );

    // Create the database directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&config.connection).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ConfigError::Database(format!("Failed to create database directory: {}", e))
        })?;
    }

    // Initialize embedded database
    let db = Surreal::new::<RocksDb>(&config.connection)
        .await
        .map_err(|e| ConfigError::Database(format!("Failed to open RocksDB: {}", e)))?;

    // Select namespace and database
    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await?;

    tracing::info!("SurrealDB initialized successfully");

    Ok(db)
}

/// Initialize a remote SurrealDB connection (for testing with Docker).
pub async fn init_remote_database(config: &DatabaseConfig) -> Result<RemoteDb, ConfigError> {
    tracing::info!(
        url = %config.connection,
        namespace = %config.namespace,
        database = %config.database,
        "Connecting to remote SurrealDB"
    );

    let db = Surreal::new::<Ws>(&config.connection)
        .await
        .map_err(|e| ConfigError::Database(format!("Failed to connect: {}", e)))?;

    // Authenticate if credentials provided
    if let (Some(user), Some(pass)) = (&config.username, &config.password) {
        db.signin(Root {
            username: user,
            password: pass,
        })
        .await
        .map_err(|e| ConfigError::Database(format!("Authentication failed: {}", e)))?;
    }

    // Select namespace and database
    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await?;

    tracing::info!("Connected to remote SurrealDB successfully");

    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert!(config.embedded);
        assert_eq!(config.connection, "./data/gateway.db");
        assert_eq!(config.namespace, "gateway");
        assert_eq!(config.database, "config");
    }

    #[test]
    fn test_embedded_config() {
        let config = DatabaseConfig::embedded("/custom/path.db");
        assert!(config.embedded);
        assert_eq!(config.connection, "/custom/path.db");
    }

    #[test]
    fn test_remote_config() {
        let config = DatabaseConfig::remote("ws://localhost:8000", "admin", "secret");
        assert!(!config.embedded);
        assert_eq!(config.connection, "ws://localhost:8000");
        assert_eq!(config.username, Some("admin".to_string()));
        assert_eq!(config.password, Some("secret".to_string()));
    }
}

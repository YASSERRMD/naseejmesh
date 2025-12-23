//! Configuration errors for the SurrealDB layer.

use thiserror::Error;

/// Errors that can occur during configuration operations
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Database connection/initialization failed
    #[error("Database error: {0}")]
    Database(String),

    /// Route not found
    #[error("Route not found: {id}")]
    RouteNotFound { id: String },

    /// Route already exists
    #[error("Route already exists: {id}")]
    RouteExists { id: String },

    /// Invalid route configuration
    #[error("Invalid route configuration: {reason}")]
    InvalidRoute { reason: String },

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Live query subscription failed
    #[error("Live query error: {0}")]
    LiveQuery(String),

    /// Watcher task error
    #[error("Watcher error: {0}")]
    Watcher(String),
}

impl From<surrealdb::Error> for ConfigError {
    fn from(err: surrealdb::Error) -> Self {
        ConfigError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::Serialization(err.to_string())
    }
}

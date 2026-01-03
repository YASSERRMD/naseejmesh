//! Route Repository for SurrealDB
//!
//! Provides CRUD operations for routes persisted in SurrealDB.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

/// Repository errors
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Route configuration stored in SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Route path (e.g., /api/users)
    pub path: String,

    /// HTTP method (GET, POST, PUT, DELETE)
    pub method: String,

    /// Upstream service URL
    pub upstream: String,

    /// Optional Rhai transformation script
    #[serde(default)]
    pub transform_script: Option<String>,

    /// Protocol type (http, mqtt, grpc)
    #[serde(default = "default_protocol")]
    pub protocol: String,

    /// Whether the route is active
    #[serde(default = "default_active")]
    pub active: bool,

    /// Who created the route (ai, user, system)
    #[serde(default = "default_creator")]
    pub created_by: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

fn default_protocol() -> String { "http".to_string() }
fn default_active() -> bool { true }
fn default_creator() -> String { "user".to_string() }

/// Route update payload
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform_script: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Route repository for SurrealDB operations
pub struct RouteRepository<C: surrealdb::Connection> {
    db: Arc<surrealdb::Surreal<C>>,
}

impl<C: surrealdb::Connection> RouteRepository<C> {
    /// Create a new route repository
    pub fn new(db: Arc<surrealdb::Surreal<C>>) -> Self {
        Self { db }
    }

    /// Generate a route ID from path and method
    fn route_id(path: &str, method: &str) -> String {
        let sanitized_path = path.replace('/', "_").trim_matches('_').to_string();
        format!("{}_{}", method.to_lowercase(), sanitized_path)
    }

    /// Create a new route
    pub async fn create(&self, route: RouteConfig) -> Result<RouteConfig, RepositoryError> {
        let route_id = Self::route_id(&route.path, &route.method);
        let log_method = route.method.clone();
        let log_path = route.path.clone();

        // Check if exists
        if self.find_by_id(&route_id).await.is_ok() {
            return Err(RepositoryError::AlreadyExists(format!(
                "{} {}",
                log_method, log_path
            )));
        }

        let created: Option<RouteConfig> = self
            .db
            .create(("routes", route_id.clone()))
            .content(route)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match created {
            Some(r) => {
                info!(path = %r.path, method = %r.method, "Route created");
                Ok(r)
            }
            None => Err(RepositoryError::DatabaseError("Create returned None".into())),
        }
    }

    /// Find route by ID
    pub async fn find_by_id(&self, id: &str) -> Result<RouteConfig, RepositoryError> {
        let route: Option<RouteConfig> = self
            .db
            .select(("routes", id))
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        route.ok_or_else(|| RepositoryError::NotFound(id.to_string()))
    }

    /// Find route by path and method
    pub async fn find_by_path_method(
        &self,
        path: &str,
        method: &str,
    ) -> Result<RouteConfig, RepositoryError> {
        let route_id = Self::route_id(path, method);
        self.find_by_id(&route_id).await
    }

    /// List all active routes
    pub async fn list_active(&self) -> Result<Vec<RouteConfig>, RepositoryError> {
        let mut result = self
            .db
            .query("SELECT * FROM routes WHERE active = true ORDER BY created_at DESC")
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let routes: Vec<RouteConfig> = result
            .take(0)
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(routes)
    }

    /// List all routes (including inactive)
    pub async fn list_all(&self) -> Result<Vec<RouteConfig>, RepositoryError> {
        let routes: Vec<RouteConfig> = self
            .db
            .select("routes")
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(routes)
    }

    /// Update a route
    pub async fn update(
        &self,
        path: &str,
        method: &str,
        updates: RouteUpdate,
    ) -> Result<RouteConfig, RepositoryError> {
        let route_id = Self::route_id(path, method);

        // Ensure route exists
        self.find_by_id(&route_id).await?;

        let mut update_data = updates;
        update_data.updated_at = Some(Utc::now());

        let updated: Option<RouteConfig> = self
            .db
            .update(("routes", &route_id))
            .merge(update_data)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match updated {
            Some(r) => {
                info!(path = %r.path, method = %r.method, "Route updated");
                Ok(r)
            }
            None => Err(RepositoryError::NotFound(route_id)),
        }
    }

    /// Delete a route
    pub async fn delete(&self, path: &str, method: &str) -> Result<(), RepositoryError> {
        let route_id = Self::route_id(path, method);

        let deleted: Option<RouteConfig> = self
            .db
            .delete(("routes", &route_id))
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if deleted.is_some() {
            info!(path = %path, method = %method, "Route deleted");
            Ok(())
        } else {
            Err(RepositoryError::NotFound(route_id))
        }
    }

    /// Deactivate a route (soft delete)
    pub async fn deactivate(&self, path: &str, method: &str) -> Result<RouteConfig, RepositoryError> {
        self.update(path, method, RouteUpdate {
            active: Some(false),
            ..Default::default()
        }).await
    }

    /// Count routes
    pub async fn count(&self) -> Result<usize, RepositoryError> {
        let routes = self.list_all().await?;
        Ok(routes.len())
    }

    /// List routes created by AI
    pub async fn list_ai_routes(&self) -> Result<Vec<RouteConfig>, RepositoryError> {
        let mut result = self
            .db
            .query("SELECT * FROM routes WHERE created_by = 'ai' ORDER BY created_at DESC")
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let routes: Vec<RouteConfig> = result
            .take(0)
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(routes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_id_generation() {
        assert_eq!(RouteRepository::<surrealdb::engine::any::Any>::route_id("/api/users", "GET"), "get_api_users");
        assert_eq!(RouteRepository::<surrealdb::engine::any::Any>::route_id("/", "POST"), "post_");
    }

    #[test]
    fn test_route_config_serialization() {
        let route = RouteConfig {
            path: "/api/test".to_string(),
            method: "GET".to_string(),
            upstream: "http://localhost:8080".to_string(),
            transform_script: None,
            protocol: "http".to_string(),
            active: true,
            created_by: "ai".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: serde_json::json!({}),
        };

        let json = serde_json::to_string(&route).unwrap();
        assert!(json.contains("api/test"));
    }
}

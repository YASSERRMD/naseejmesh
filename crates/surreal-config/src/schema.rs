//! Route schema and CRUD operations for SurrealDB.
//!
//! This module provides the data access layer for route configuration,
//! with operations that work with both embedded and remote SurrealDB.

use gateway_core::config::Route;
use surrealdb::Connection;
use surrealdb::Surreal;
use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

/// Table name for routes
const ROUTES_TABLE: &str = "routes";

/// Internal database representation of a Route to handle SurrealDB v2 RecordIDs
#[derive(Debug, Serialize, Deserialize)]
struct DbRoute {
    id: surrealdb::sql::Thing,
    name: String,
    path: String,
    upstream: String,
    methods: Vec<String>,
    active: bool,
    weight: u32,
    timeout_ms: u64,
    retries: u32,
    load_balancer: String,
    description: String,
    permissions: Vec<String>,
    created_at: String,
}

impl From<DbRoute> for Route {
    fn from(db: DbRoute) -> Self {
        Route {
            id: db.id.to_string(),
            name: db.name,
            path: db.path,
            upstream: db.upstream,
            methods: db.methods,
            active: db.active,
            weight: db.weight,
            timeout_ms: db.timeout_ms,
            retries: db.retries,
            load_balancer: db.load_balancer,
            description: db.description,
            permissions: db.permissions,
            created_at: db.created_at,
        }
    }
}

/// Create a new route in the database.
pub async fn create_route<C: Connection>(db: &Surreal<C>, route: Route) -> Result<Route, ConfigError> {
    validate_route(&route)?;

    tracing::debug!(id = %route.id, path = %route.path, "Creating route");

    let id = route.id.clone();
    let mut result = db
        .query("CREATE type::thing('routes', $id) SET name = $name, path = $path, upstream = $upstream, methods = $methods, active = $active, weight = $weight, timeout_ms = $timeout, retries = $retries, load_balancer = $lb, description = $desc, permissions = $perm, created_at = $now")
        .bind(("id", id))
        .bind(("name", route.name))
        .bind(("path", route.path))
        .bind(("upstream", route.upstream))
        .bind(("methods", route.methods))
        .bind(("active", route.active))
        .bind(("weight", route.weight))
        .bind(("timeout", route.timeout_ms))
        .bind(("retries", route.retries))
        .bind(("lb", route.load_balancer))
        .bind(("desc", route.description))
        .bind(("perm", route.permissions))
        .bind(("now", route.created_at))
        .await?;
    
    let created: Option<DbRoute> = result.take(0)?;
    created.map(Route::from).ok_or_else(|| ConfigError::Database("Failed to create route".to_string()))
}

/// Get a specific route by ID.
pub async fn get_route<C: Connection>(db: &Surreal<C>, id: &str) -> Result<Option<Route>, ConfigError> {
    let route: Option<DbRoute> = db.select((ROUTES_TABLE, id)).await?;
    Ok(route.map(Route::from))
}

/// Get all routes from the database.
pub async fn get_all_routes<C: Connection>(db: &Surreal<C>) -> Result<Vec<Route>, ConfigError> {
    let routes: Vec<DbRoute> = db.select(ROUTES_TABLE).await?;
    Ok(routes.into_iter().map(Route::from).collect())
}

/// Update an existing route.
pub async fn update_route<C: Connection>(db: &Surreal<C>, route: Route) -> Result<Route, ConfigError> {
    validate_route(&route)?;

    tracing::debug!(id = %route.id, path = %route.path, "Updating route");

    let id = route.id.clone();
    let mut result = db
        .query("UPDATE type::thing('routes', $id) SET name = $name, path = $path, upstream = $upstream, methods = $methods, active = $active, weight = $weight, timeout_ms = $timeout, retries = $retries, load_balancer = $lb, description = $desc, permissions = $perm")
        .bind(("id", id.clone()))
        .bind(("name", route.name))
        .bind(("path", route.path))
        .bind(("upstream", route.upstream))
        .bind(("methods", route.methods))
        .bind(("active", route.active))
        .bind(("weight", route.weight))
        .bind(("timeout", route.timeout_ms))
        .bind(("retries", route.retries))
        .bind(("lb", route.load_balancer))
        .bind(("desc", route.description))
        .bind(("perm", route.permissions))
        .await?;

    let updated: Option<DbRoute> = result.take(0)?;
    updated.map(Route::from).ok_or_else(|| ConfigError::RouteNotFound { id })
}

/// Delete a route by ID.
pub async fn delete_route<C: Connection>(db: &Surreal<C>, id: &str) -> Result<(), ConfigError> {
    tracing::debug!(id = %id, "Deleting route");

    let deleted: Option<DbRoute> = db.delete((ROUTES_TABLE, id)).await?;

    if deleted.is_none() {
        return Err(ConfigError::RouteNotFound { id: id.to_string() });
    }

    Ok(())
}

/// Get the count of active routes.
pub async fn count_active_routes<C: Connection>(db: &Surreal<C>) -> Result<usize, ConfigError> {
    let routes: Vec<DbRoute> = db.select(ROUTES_TABLE).await?;
    Ok(routes.iter().filter(|r| r.active).count())
}

/// Bulk insert routes (useful for initial seeding).
pub async fn bulk_create_routes<C: Connection>(db: &Surreal<C>, routes: Vec<Route>) -> Result<usize, ConfigError> {
    let mut created = 0;

    for route in routes {
        if let Err(e) = create_route(db, route).await {
            tracing::warn!(error = %e, "Failed to create route during bulk insert");
        } else {
            created += 1;
        }
    }

    Ok(created)
}

/// Validate a route configuration.
fn validate_route(route: &Route) -> Result<(), ConfigError> {
    if route.id.is_empty() {
        return Err(ConfigError::InvalidRoute {
            reason: "Route ID cannot be empty".to_string(),
        });
    }

    if route.path.is_empty() {
        return Err(ConfigError::InvalidRoute {
            reason: "Route path cannot be empty".to_string(),
        });
    }

    if !route.path.starts_with('/') {
        return Err(ConfigError::InvalidRoute {
            reason: "Route path must start with '/'".to_string(),
        });
    }

    if route.upstream.is_empty() {
        return Err(ConfigError::InvalidRoute {
            reason: "Upstream URL cannot be empty".to_string(),
        });
    }

    // Validate upstream URL format
    if !route.upstream.starts_with("http://") && !route.upstream.starts_with("https://") {
        return Err(ConfigError::InvalidRoute {
            reason: "Upstream must be a valid HTTP/HTTPS URL".to_string(),
        });
    }

    Ok(())
}

/// Create a default route (helper for seed_default_routes)
fn default_route(id: &str, path: &str, upstream: &str, description: &str) -> Route {
    Route {
        id: id.to_string(),
        name: id.to_string(),
        path: path.to_string(),
        upstream: upstream.to_string(),
        weight: 100,
        active: true,
        methods: Vec::new(),
        timeout_ms: 30000,
        retries: 3,
        load_balancer: "round_robin".to_string(),
        description: description.to_string(),
        permissions: Vec::new(),
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Seed default routes for development/testing.
pub async fn seed_default_routes<C: Connection>(db: &Surreal<C>) -> Result<(), ConfigError> {
    let defaults = vec![
        Route::new("health", "/_gateway/health", "http://localhost:8080"),
        Route::new("ready", "/_gateway/ready", "http://localhost:8080"),
        default_route(
            "api-catchall",
            "/api/*",
            "http://localhost:3000",
            "Default API catch-all route",
        ),
    ];

    for route in defaults {
        // Only create if doesn't exist
        if get_route(db, &route.id).await?.is_none() {
            create_route(db, route).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_route(id: &str, path: &str, upstream: &str) -> Route {
        Route {
            id: id.to_string(),
            name: id.to_string(),
            path: path.into(),
            upstream: upstream.into(),
            weight: 100,
            active: true,
            methods: Vec::new(),
            timeout_ms: 30000,
            retries: 0,
            load_balancer: "round_robin".to_string(),
            description: String::new(),
            permissions: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_validate_empty_id() {
        let route = test_route("", "/test", "http://localhost:8080");
        let result = validate_route(&route);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_path() {
        let route = test_route("test", "", "http://localhost:8080");
        let result = validate_route(&route);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_path() {
        let route = test_route("test", "no-leading-slash", "http://localhost:8080");
        let result = validate_route(&route);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_upstream() {
        let route = test_route("test", "/test", "invalid-url");
        let result = validate_route(&route);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_route() {
        let route = Route::new("test", "/api/users", "http://user-service:8080");
        assert!(validate_route(&route).is_ok());
    }
}

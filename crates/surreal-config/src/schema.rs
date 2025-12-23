//! Route schema and CRUD operations for SurrealDB.
//!
//! This module provides the data access layer for route configuration,
//! with operations that work with both embedded and remote SurrealDB.

use gateway_core::config::Route;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

use crate::error::ConfigError;

/// Table name for routes
const ROUTES_TABLE: &str = "routes";

/// Create a new route in the database.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `route` - Route configuration to create
///
/// # Returns
///
/// The created route with any server-generated fields
pub async fn create_route(db: &Surreal<Db>, route: Route) -> Result<Route, ConfigError> {
    // Validate route
    validate_route(&route)?;

    tracing::debug!(id = %route.id, path = %route.path, "Creating route");

    let id = route.id.clone();
    let created: Option<Route> = db
        .create((ROUTES_TABLE, &id))
        .content(route)
        .await?;

    created.ok_or_else(|| ConfigError::Database("Failed to create route".to_string()))
}

/// Get a specific route by ID.
pub async fn get_route(db: &Surreal<Db>, id: &str) -> Result<Option<Route>, ConfigError> {
    let route: Option<Route> = db.select((ROUTES_TABLE, id)).await?;
    Ok(route)
}

/// Get all routes from the database.
pub async fn get_all_routes(db: &Surreal<Db>) -> Result<Vec<Route>, ConfigError> {
    let routes: Vec<Route> = db.select(ROUTES_TABLE).await?;
    Ok(routes)
}

/// Update an existing route.
pub async fn update_route(db: &Surreal<Db>, route: Route) -> Result<Route, ConfigError> {
    // Validate route
    validate_route(&route)?;

    tracing::debug!(id = %route.id, path = %route.path, "Updating route");

    let id = route.id.clone();
    let updated: Option<Route> = db
        .update((ROUTES_TABLE, &id))
        .content(route)
        .await?;

    updated.ok_or_else(|| ConfigError::RouteNotFound { id })
}

/// Delete a route by ID.
pub async fn delete_route(db: &Surreal<Db>, id: &str) -> Result<(), ConfigError> {
    tracing::debug!(id = %id, "Deleting route");

    let deleted: Option<Route> = db.delete((ROUTES_TABLE, id)).await?;

    if deleted.is_none() {
        return Err(ConfigError::RouteNotFound { id: id.to_string() });
    }

    Ok(())
}

/// Get the count of active routes.
pub async fn count_active_routes(db: &Surreal<Db>) -> Result<usize, ConfigError> {
    let routes: Vec<Route> = db.select(ROUTES_TABLE).await?;
    Ok(routes.iter().filter(|r| r.active).count())
}

/// Bulk insert routes (useful for initial seeding).
pub async fn bulk_create_routes(db: &Surreal<Db>, routes: Vec<Route>) -> Result<usize, ConfigError> {
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
        path: path.to_string(),
        upstream: upstream.to_string(),
        weight: 100,
        active: true,
        methods: Vec::new(),
        timeout_ms: 30000,
        description: description.to_string(),
    }
}

/// Seed default routes for development/testing.
pub async fn seed_default_routes(db: &Surreal<Db>) -> Result<(), ConfigError> {
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
            path: path.to_string(),
            upstream: upstream.to_string(),
            weight: 100,
            active: true,
            methods: Vec::new(),
            timeout_ms: 30000,
            description: String::new(),
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

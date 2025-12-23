//! Configuration watcher with Live Query support.
//!
//! This module implements the reactive configuration update mechanism using
//! SurrealDB's Live Query feature. Changes to the routes table are detected
//! in real-time and propagated to the gateway's routing table via ArcSwap.

use std::sync::Arc;

use arc_swap::ArcSwap;
use futures::StreamExt;
use gateway_core::config::RouterMap;
use gateway_core::router::build_router_map;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use gateway_core::config::Route;

use crate::error::ConfigError;
use crate::schema::get_all_routes;

/// Start the configuration watcher task.
///
/// This function subscribes to the routes table using SurrealDB's Live Query
/// feature and updates the shared routing configuration whenever changes occur.
///
/// # Strategy: Full Reload
///
/// Instead of processing individual diff events, we perform a full table reload
/// on each notification. This approach:
/// - Ensures idempotency (missed events are auto-corrected)
/// - Simplifies the update logic
/// - Allows atomic replacement via ArcSwap
///
/// # Arguments
///
/// * `db` - SurrealDB connection
/// * `config` - Shared routing configuration wrapped in ArcSwap
///
/// # Note
///
/// This function runs indefinitely and should be spawned as a background task.
pub async fn start_config_watcher(
    db: Surreal<Db>,
    config: Arc<ArcSwap<RouterMap>>,
) -> Result<(), ConfigError> {
    tracing::info!("Starting configuration watcher with Live Query subscription");

    // Perform initial configuration load
    reload_config(&db, &config).await?;

    // Subscribe to route changes using Live Query
    // SurrealDB 2.x uses a different API for live queries
    let mut stream = db
        .query("LIVE SELECT * FROM routes")
        .await
        .map_err(|e| ConfigError::LiveQuery(e.to_string()))?
        .stream::<surrealdb::Notification<Route>>(0)
        .map_err(|e| ConfigError::LiveQuery(e.to_string()))?;

    tracing::info!("Live Query subscription established on 'routes' table");

    // Process incoming notifications
    while let Some(notification) = stream.next().await {
        match notification {
            Ok(event) => {
                tracing::info!(
                    action = ?event.action,
                    "Configuration change detected"
                );

                // Full reload strategy: fetch all routes and rebuild map
                if let Err(e) = reload_config(&db, &config).await {
                    tracing::error!(error = %e, "Failed to reload configuration");
                    // Continue watching - don't exit on reload failure
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Error receiving Live Query notification");
                // Continue watching - transient errors shouldn't stop the watcher
            }
        }
    }

    tracing::warn!("Live Query stream ended unexpectedly");
    Err(ConfigError::Watcher("Live Query stream ended".to_string()))
}

/// Reload the entire routing configuration from the database.
///
/// This function:
/// 1. Fetches all routes from SurrealDB
/// 2. Builds an optimized routing map
/// 3. Atomically swaps the new map into the shared configuration
///
/// The atomic swap ensures that in-flight requests continue using the old
/// configuration while new requests pick up the updated routes.
async fn reload_config(
    db: &Surreal<Db>,
    config: &Arc<ArcSwap<RouterMap>>,
) -> Result<(), ConfigError> {
    // Fetch all routes from database
    let routes = get_all_routes(db).await?;
    let route_count = routes.len();
    let active_count = routes.iter().filter(|r| r.active).count();

    // Build optimized routing map (filters inactive routes)
    let new_map = build_router_map(routes);

    // Atomic swap - wait-free for readers
    config.store(Arc::new(new_map));

    tracing::info!(
        total_routes = route_count,
        active_routes = active_count,
        "Configuration reloaded successfully"
    );

    Ok(())
}

/// Manually trigger a configuration reload.
///
/// Useful for administrative operations or recovery scenarios.
pub async fn force_reload(
    db: &Surreal<Db>,
    config: &Arc<ArcSwap<RouterMap>>,
) -> Result<(), ConfigError> {
    tracing::info!("Forcing configuration reload");
    reload_config(db, config).await
}

/// Get statistics about the current configuration.
#[derive(Debug, Clone)]
pub struct ConfigStats {
    pub total_routes: usize,
    pub active_routes: usize,
    pub unique_upstreams: usize,
}

pub fn get_config_stats(config: &Arc<ArcSwap<RouterMap>>) -> ConfigStats {
    let map = config.load();
    let unique_upstreams: std::collections::HashSet<_> =
        map.values().map(|r| &r.upstream).collect();

    ConfigStats {
        total_routes: map.len(),
        active_routes: map.len(), // All routes in map are active
        unique_upstreams: unique_upstreams.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_config_stats_empty() {
        let config = Arc::new(ArcSwap::from_pointee(HashMap::new()));
        let stats = get_config_stats(&config);

        assert_eq!(stats.total_routes, 0);
        assert_eq!(stats.active_routes, 0);
        assert_eq!(stats.unique_upstreams, 0);
    }

    #[test]
    fn test_config_stats_with_routes() {
        let mut map = HashMap::new();
        map.insert(
            "/api/users".to_string(),
            Route::new("1", "/api/users", "http://service-a:8080"),
        );
        map.insert(
            "/api/posts".to_string(),
            Route::new("2", "/api/posts", "http://service-b:8080"),
        );
        map.insert(
            "/api/comments".to_string(),
            Route::new("3", "/api/comments", "http://service-a:8080"), // Same upstream
        );

        let config = Arc::new(ArcSwap::from_pointee(map));
        let stats = get_config_stats(&config);

        assert_eq!(stats.total_routes, 3);
        assert_eq!(stats.active_routes, 3);
        assert_eq!(stats.unique_upstreams, 2); // service-a and service-b
    }
}

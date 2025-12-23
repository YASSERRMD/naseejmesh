//! Router implementation with path matching and route lookup.
//!
//! Provides efficient route matching with exact match priority followed
//! by longest-prefix matching. Designed to work with ArcSwap for
//! wait-free reads.

use crate::config::{Route, RouterMap};


/// Build an optimized router map from a list of routes.
///
/// Only active routes are included in the map. The map is keyed by
/// the exact path pattern for O(1) exact-match lookups.
pub fn build_router_map(routes: Vec<Route>) -> RouterMap {
    routes
        .into_iter()
        .filter(|r| r.active)
        .map(|r| (r.path.clone(), r))
        .collect()
}

/// Match a request path against the routing table.
///
/// Matching strategy:
/// 1. Exact match (O(1) HashMap lookup)
/// 2. Longest prefix match (O(n) scan, but typically small n)
///
/// Returns `None` if no route matches.
pub fn match_route<'a>(path: &str, map: &'a RouterMap) -> Option<&'a Route> {
    // Try exact match first (fast path)
    if let Some(route) = map.get(path) {
        return Some(route);
    }

    // Fall back to prefix matching
    // Find the longest matching prefix
    map.iter()
        .filter(|(pattern, _)| path_matches(path, pattern))
        .max_by_key(|(pattern, _)| pattern.len())
        .map(|(_, route)| route)
}

/// Check if a path matches a pattern.
///
/// Currently supports:
/// - Exact match: "/api/users" matches "/api/users"
/// - Prefix match: "/api/" matches "/api/users", "/api/posts"
/// - Wildcard suffix: "/api/*" matches "/api/anything"
fn path_matches(path: &str, pattern: &str) -> bool {
    // Wildcard pattern
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return path.starts_with(prefix) && (path.len() == prefix.len() || path[prefix.len()..].starts_with('/'));
    }

    // Prefix pattern (ends with /)
    if pattern.ends_with('/') {
        return path.starts_with(pattern) || path == pattern.trim_end_matches('/');
    }

    // Exact match
    path == pattern
}

/// Statistics about the current routing table
#[derive(Debug, Clone, Default)]
pub struct RouterStats {
    /// Total number of routes in the table
    pub total_routes: usize,
    /// Number of active routes
    pub active_routes: usize,
    /// Number of unique upstream services
    pub unique_upstreams: usize,
}

/// Calculate statistics about a router map
pub fn router_stats(map: &RouterMap) -> RouterStats {
    let unique_upstreams: std::collections::HashSet<_> =
        map.values().map(|r| &r.upstream).collect();

    RouterStats {
        total_routes: map.len(),
        active_routes: map.len(), // All routes in map are active (filtered during build)
        unique_upstreams: unique_upstreams.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_routes() -> Vec<Route> {
        vec![
            Route::new("1", "/api/users", "http://user-service:8080"),
            Route::new("2", "/api/posts", "http://post-service:8080"),
            Route::new("3", "/api/v2/", "http://v2-service:8080"),
            Route::new("4", "/health", "http://localhost:8080"),
            {
                let mut r = Route::new("5", "/disabled", "http://disabled:8080");
                r.active = false;
                r
            },
            Route::new("6", "/api/*", "http://api-catchall:8080"),
        ]
    }

    #[test]
    fn test_build_router_map_filters_inactive() {
        let routes = create_test_routes();
        let map = build_router_map(routes);

        assert_eq!(map.len(), 5); // 6 routes - 1 inactive
        assert!(!map.contains_key("/disabled"));
    }

    #[test]
    fn test_exact_match() {
        let routes = create_test_routes();
        let map = build_router_map(routes);

        let route = match_route("/api/users", &map).unwrap();
        assert_eq!(route.upstream, "http://user-service:8080");

        let route = match_route("/health", &map).unwrap();
        assert_eq!(route.upstream, "http://localhost:8080");
    }

    #[test]
    fn test_prefix_match() {
        let routes = create_test_routes();
        let map = build_router_map(routes);

        // /api/v2/ should match /api/v2/anything
        let route = match_route("/api/v2/resources", &map).unwrap();
        assert_eq!(route.upstream, "http://v2-service:8080");
    }

    #[test]
    fn test_wildcard_match() {
        let routes = create_test_routes();
        let map = build_router_map(routes);

        // /api/* should match /api/unknown but not exact /api/users
        // Since /api/users is an exact match, it takes priority
        let route = match_route("/api/users", &map).unwrap();
        assert_eq!(route.upstream, "http://user-service:8080");

        // Unknown path falls back to wildcard
        let route = match_route("/api/unknown/deep/path", &map).unwrap();
        assert_eq!(route.upstream, "http://api-catchall:8080");
    }

    #[test]
    fn test_no_match() {
        let routes = create_test_routes();
        let map = build_router_map(routes);

        assert!(match_route("/unknown", &map).is_none());
        assert!(match_route("/", &map).is_none());
    }

    #[test]
    fn test_router_stats() {
        let routes = create_test_routes();
        let map = build_router_map(routes);
        let stats = router_stats(&map);

        assert_eq!(stats.total_routes, 5);
        assert_eq!(stats.active_routes, 5);
        assert_eq!(stats.unique_upstreams, 5);
    }
}

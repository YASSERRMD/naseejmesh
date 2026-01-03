//! Route configuration types for the API Gateway.
//!
//! Defines the core data structures for routing rules, designed to be
//! serializable for SurrealDB storage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::serde_utils::deserialize_id;

/// A single routing rule mapping a path to an upstream service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Route {
    /// Unique identifier for the route
    pub id: String,




    /// Human-readable name
    #[serde(default)]
    pub name: String,

    /// URL path pattern to match (e.g., "/api/v1/users")
    pub path: String,

    /// Upstream configuration
    pub upstream: String,

    /// HTTP methods allowed (empty = all methods)
    #[serde(default)]
    pub methods: Vec<String>,

    /// Whether this route is active
    #[serde(default = "default_active")]
    pub active: bool,

    /// Traffic weight for load balancing (0-100)
    #[serde(default = "default_weight")]
    pub weight: u32,

    /// Request timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Number of retries for failed requests
    #[serde(default)]
    pub retries: u32,

    /// Load balancing strategy
    #[serde(default = "default_lb")]
    pub load_balancer: String,

    /// Optional description for documentation
    #[serde(default)]
    pub description: String,

    /// Permissions required to access this route
    #[serde(default)]
    pub permissions: Vec<String>,

    /// Created timestamp
    #[serde(default = "default_timestamp")]
    pub created_at: String,
}

fn default_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn default_lb() -> String {
    "round_robin".to_string()
}


fn default_weight() -> u32 {
    100
}

fn default_active() -> bool {
    true
}

fn default_timeout() -> u64 {
    30000 // 30 seconds
}

impl Route {
    /// Create a new route with minimal required fields
    pub fn new(id: impl Into<String>, path: impl Into<String>, upstream: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            name: id_str,
            path: path.into(),
            upstream: upstream.into(),
            weight: default_weight(),
            active: default_active(),
            methods: Vec::new(),
            timeout_ms: default_timeout(),
            retries: 0,
            load_balancer: default_lb(),
            description: String::new(),
            permissions: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Check if a specific HTTP method is allowed
    pub fn allows_method(&self, method: &str) -> bool {
        self.methods.is_empty() || self.methods.iter().any(|m| m.eq_ignore_ascii_case(method))
    }
}

/// The routing table - a map from path patterns to routes.
/// This structure is designed to be swapped atomically via ArcSwap.
pub type RouterMap = HashMap<String, Route>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_creation() {
        let route = Route::new("test-1", "/api/users", "http://localhost:3000");
        assert_eq!(route.id, "test-1");
        assert_eq!(route.path, "/api/users");
        assert_eq!(route.upstream, "http://localhost:3000");
        assert!(route.active);
        assert_eq!(route.weight, 100);
    }

    #[test]
    fn test_method_filtering() {
        let mut route = Route::new("test-1", "/api/users", "http://localhost:3000");
        
        // Empty methods = all allowed
        assert!(route.allows_method("GET"));
        assert!(route.allows_method("POST"));
        
        // Specific methods
        route.methods = vec!["GET".to_string(), "POST".to_string()];
        assert!(route.allows_method("GET"));
        assert!(route.allows_method("post")); // case insensitive
        assert!(!route.allows_method("DELETE"));
    }
}

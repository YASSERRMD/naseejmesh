//! Request handler implementation using service_fn pattern.
//!
//! This module implements the core request processing logic with the
//! "Clone-and-Move" pattern required for async service handlers.

use std::convert::Infallible;
use std::sync::Arc;

use arc_swap::ArcSwap;
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};

use crate::config::RouterMap;
use crate::error::GatewayError;
use crate::router::match_route;

/// Handle an incoming HTTP request.
///
/// This is the main entry point for request processing. It performs:
/// 1. Route matching against the current configuration
/// 2. Method validation
/// 3. Request forwarding preparation (Phase 2 will add actual forwarding)
///
/// # Arguments
///
/// * `req` - The incoming HTTP request
/// * `config` - ArcSwap-wrapped routing configuration for wait-free access
///
/// # Returns
///
/// Always returns `Ok(Response)` - errors are converted to HTTP error responses.
pub async fn handle_request(
    req: Request<Incoming>,
    config: Arc<ArcSwap<RouterMap>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // Load current configuration (wait-free read)
    let router_map = config.load();

    // Extract request details
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let _headers = req.headers().clone();

    tracing::debug!(
        method = %method,
        path = %path,
        "Processing request"
    );

    // Match route
    let result = match match_route(&path, &router_map) {
        Some(route) => {
            // Check method
            if !route.allows_method(method.as_str()) {
                Err(GatewayError::MethodNotAllowed {
                    method: method.to_string(),
                    path: path.clone(),
                })
            } else {
                // Route matched - in Phase 1, we return a stub response
                // Phase 2 will implement actual upstream forwarding
                Ok(build_stub_response(&route.upstream, &path, &method))
            }
        }
        None => Err(GatewayError::RouteNotFound { path: path.clone() }),
    };

    // Convert result to HTTP response
    let response = match result {
        Ok(resp) => resp,
        Err(e) => build_error_response(e),
    };

    Ok(response)
}

/// Build a stub response indicating successful routing.
///
/// In Phase 1, we don't actually forward to upstream services.
/// This response confirms the route was matched successfully.
fn build_stub_response(upstream: &str, path: &str, method: &Method) -> Response<Full<Bytes>> {
    let body = serde_json::json!({
        "status": "routed",
        "upstream": upstream,
        "path": path,
        "method": method.as_str(),
        "message": "Phase 1: Route matched. Upstream forwarding will be implemented in Phase 2."
    });

    let body_bytes = Bytes::from(serde_json::to_vec(&body).unwrap_or_default());

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("X-Gateway-Phase", "1")
        .header("X-Gateway-Upstream", upstream)
        .body(Full::new(body_bytes))
        .unwrap()
}

/// Build an error response from a GatewayError.
fn build_error_response(error: GatewayError) -> Response<Full<Bytes>> {
    let status_code = error.status_code();
    let category = error.category();

    tracing::warn!(
        error = %error,
        status = status_code,
        category = category,
        "Request failed"
    );

    let body = serde_json::json!({
        "error": {
            "message": error.to_string(),
            "category": category,
            "retryable": error.is_retryable()
        }
    });

    let body_bytes = Bytes::from(serde_json::to_vec(&body).unwrap_or_default());

    Response::builder()
        .status(StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .header("Content-Type", "application/json")
        .header("X-Gateway-Error-Category", category)
        .body(Full::new(body_bytes))
        .unwrap()
}

/// Health check handler for the gateway.
pub fn health_check() -> Response<Full<Bytes>> {
    let body = serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    });

    let body_bytes = Bytes::from(serde_json::to_vec(&body).unwrap_or_default());

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(body_bytes))
        .unwrap()
}

/// Readiness check - confirms the gateway has loaded configuration.
pub fn readiness_check(config: &Arc<ArcSwap<RouterMap>>) -> Response<Full<Bytes>> {
    let router_map = config.load();
    let route_count = router_map.len();

    let (status, ready) = if route_count > 0 {
        (StatusCode::OK, true)
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, false)
    };

    let body = serde_json::json!({
        "ready": ready,
        "routes_loaded": route_count,
    });

    let body_bytes = Bytes::from(serde_json::to_vec(&body).unwrap_or_default());

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(body_bytes))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Route;
    use std::collections::HashMap;

    fn create_test_config() -> Arc<ArcSwap<RouterMap>> {
        let mut map = HashMap::new();
        map.insert(
            "/api/users".to_string(),
            Route::new("1", "/api/users", "http://user-service:8080"),
        );
        map.insert(
            "/api/posts".to_string(),
            {
                let mut r = Route::new("2", "/api/posts", "http://post-service:8080");
                r.methods = vec!["GET".to_string(), "POST".to_string()];
                r
            },
        );
        Arc::new(ArcSwap::from_pointee(map))
    }

    #[test]
    fn test_create_config() {
        // Verify config creation works
        let config = create_test_config();
        let map = config.load();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("/api/users"));
        assert!(map.contains_key("/api/posts"));
    }

    #[test]
    fn test_health_check() {
        let response = health_check();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_readiness_with_routes() {
        let config = create_test_config();
        let response = readiness_check(&config);
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_readiness_without_routes() {
        let config = Arc::new(ArcSwap::from_pointee(HashMap::new()));
        let response = readiness_check(&config);
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}

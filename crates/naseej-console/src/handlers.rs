//! Request handlers for the console API

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{sse::Event, Sse},
    Json,
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use gateway_core::{simulate, validate_script};

use crate::state::{AppState, RouteInfo, TransformationInfo, SecurityEvent, SchemaInfo};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateRouteRequest {
    pub path: String,
    pub upstream: String,
    #[serde(default = "default_method")]
    pub method: String,
    pub transform_script: Option<String>,
}

fn default_method() -> String {
    "GET".to_string()
}

#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    pub script: String,
    pub input: String,
}

#[derive(Debug, Serialize)]
pub struct SimulateResponse {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub execution_us: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub script: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
}

#[derive(Debug, Serialize)]
pub struct GatewayState {
    pub routes: Vec<RouteInfo>,
    pub uptime_seconds: u64,
    pub version: String,
}

/// Gateway status response matching frontend types
#[derive(Debug, Serialize)]
pub struct GatewayStatus {
    pub healthy: bool,
    pub version: String,
    pub uptime: u64,
    pub routes: usize,
    pub requests: RequestMetrics,
}

#[derive(Debug, Serialize)]
pub struct RequestMetrics {
    pub total: u64,
    #[serde(rename = "perSecond")]
    pub per_second: u64,
    #[serde(rename = "avgLatencyMs")]
    pub avg_latency_ms: u64,
    #[serde(rename = "errorRate")]
    pub error_rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct SecurityEventsQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

// ============================================================================
// Handlers
// ============================================================================

/// Gateway status endpoint - /api/status
pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Json<GatewayStatus> {
    let routes = state.routes.read().await;
    let total_requests: u64 = routes.iter().map(|r| r.requests).sum();
    let avg_latency: u64 = if routes.is_empty() {
        0
    } else {
        routes.iter().map(|r| r.avg_latency_ms).sum::<u64>() / routes.len() as u64
    };

    Json(GatewayStatus {
        healthy: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: state.uptime_seconds(),
        routes: routes.len(),
        requests: RequestMetrics {
            total: total_requests,
            per_second: 127, // Mock value - would come from metrics collector
            avg_latency_ms: avg_latency,
            error_rate: 0.02,
        },
    })
}

/// Metrics endpoint - /api/metrics
pub async fn get_metrics(
    State(state): State<Arc<AppState>>,
) -> Json<RequestMetrics> {
    let routes = state.routes.read().await;
    let total_requests: u64 = routes.iter().map(|r| r.requests).sum();
    let avg_latency: u64 = if routes.is_empty() {
        0
    } else {
        routes.iter().map(|r| r.avg_latency_ms).sum::<u64>() / routes.len() as u64
    };

    Json(RequestMetrics {
        total: total_requests,
        per_second: 127,
        avg_latency_ms: avg_latency,
        error_rate: 0.02,
    })
}

/// List all routes
pub async fn list_routes(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<RouteInfo>> {
    let routes = state.routes.read().await;
    Json(routes.clone())
}

/// Create a new route
pub async fn create_route(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateRouteRequest>,
) -> Result<Json<RouteInfo>, (StatusCode, String)> {
    // Validate transform script if provided
    if let Some(script) = &request.transform_script {
        if let Err(e) = validate_script(script) {
            return Err((StatusCode::BAD_REQUEST, format!("Invalid script: {}", e)));
        }
    }

    let route = RouteInfo {
        id: uuid::Uuid::new_v4().to_string(),
        path: request.path,
        upstream: request.upstream,
        method: request.method,
        transform_script: request.transform_script,
        active: true,
        created_at: chrono::Utc::now().to_rfc3339(),
        requests: 0,
        avg_latency_ms: 0,
    };

    let mut routes = state.routes.write().await;
    routes.push(route.clone());

    info!(route_id = %route.id, "Created new route");
    Ok(Json(route))
}

/// List transformations - /api/transformations
pub async fn list_transformations(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<TransformationInfo>> {
    let transformations = state.transformations.read().await;
    Json(transformations.clone())
}

/// List security events - /api/security/events
pub async fn list_security_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SecurityEventsQuery>,
) -> Json<Vec<SecurityEvent>> {
    let events = state.security_events.read().await;
    let limited: Vec<_> = events.iter().take(params.limit).cloned().collect();
    Json(limited)
}

/// List schemas - /api/schemas
pub async fn list_schemas(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<SchemaInfo>> {
    let schemas = state.schemas.read().await;
    Json(schemas.clone())
}

/// Simulate a transformation (dry-run)
pub async fn simulate_transform(
    Json(request): Json<SimulateRequest>,
) -> Json<SimulateResponse> {
    match simulate(&request.script, &request.input) {
        Ok(result) => Json(SimulateResponse {
            success: true,
            output: Some(result.output),
            error: None,
            execution_us: Some(result.execution_us),
        }),
        Err(e) => Json(SimulateResponse {
            success: false,
            output: None,
            error: Some(e.to_string()),
            execution_us: None,
        }),
    }
}

/// Validate a transformation script
pub async fn validate_transform(
    Json(request): Json<ValidateRequest>,
) -> Json<ValidateResponse> {
    match validate_script(&request.script) {
        Ok(()) => Json(ValidateResponse {
            valid: true,
            error: None,
        }),
        Err(e) => Json(ValidateResponse {
            valid: false,
            error: Some(e.to_string()),
        }),
    }
}

/// Chat with the AI Architect
pub async fn chat(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    let mut architect = state.architect.write().await;
    
    match architect.chat(&request.message).await {
        Ok(response) => Ok(Json(ChatResponse { response })),
        Err(e) => {
            error!(error = %e, "Chat error");
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// SSE stream for chat responses
pub async fn chat_stream(
    State(_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Create a simple SSE stream that sends heartbeats
    // In production, this would stream AI responses
    let stream = stream::unfold(0, |count| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let event = Event::default()
            .event("heartbeat")
            .data(format!(r#"{{"count": {}}}"#, count));
        Some((Ok(event), count + 1))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}

/// Get gateway state
pub async fn get_state(
    State(state): State<Arc<AppState>>,
) -> Json<GatewayState> {
    let routes = state.routes.read().await;
    
    Json(GatewayState {
        routes: routes.clone(),
        uptime_seconds: state.uptime_seconds(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simulate_valid_script() {
        let request = SimulateRequest {
            script: r#"output = upper(input);"#.to_string(),
            input: "hello".to_string(),
        };

        let response = simulate_transform(Json(request)).await;
        assert!(response.success);
        assert_eq!(response.output, Some("HELLO".to_string()));
    }

    #[tokio::test]
    async fn test_simulate_invalid_script() {
        let request = SimulateRequest {
            script: r#"let x = ;"#.to_string(),
            input: "hello".to_string(),
        };

        let response = simulate_transform(Json(request)).await;
        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_validate_valid_script() {
        let request = ValidateRequest {
            script: r#"let x = 1 + 2;"#.to_string(),
        };

        let response = validate_transform(Json(request)).await;
        assert!(response.valid);
    }

    #[tokio::test]
    async fn test_validate_invalid_script() {
        let request = ValidateRequest {
            script: r#"let x = ;"#.to_string(),
        };

        let response = validate_transform(Json(request)).await;
        assert!(!response.valid);
    }
}

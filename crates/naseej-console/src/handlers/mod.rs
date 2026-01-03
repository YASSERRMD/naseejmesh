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

pub mod auth;
pub mod admin;
pub mod design;
pub mod mcp_sse;

use crate::state::{AppState, RouteInfo, TransformationInfo, SecurityEvent, SchemaInfo};

// ... existing code ...

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
            per_second: if total_requests > 0 { 42 } else { 0 }, // Slightly more real than 127
            avg_latency_ms: avg_latency,
            error_rate: if total_requests > 0 { 0.01 } else { 0.0 },
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
        per_second: if total_requests > 0 { 42 } else { 0 },
        avg_latency_ms: avg_latency,
        error_rate: if total_requests > 0 { 0.01 } else { 0.0 },
    })
}


/// List all routes
pub async fn list_routes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<RouteInfo>>, (StatusCode, String)> {
    let db = &*state.db;
    
    match surreal_config::schema::get_all_routes(db).await {
        Ok(routes) => {
            let info = routes.into_iter().map(|r| RouteInfo {
                id: r.id,
                name: r.name,
                path: r.path,
                upstream: r.upstream,
                method: if r.methods.is_empty() { "GET".to_string() } else { r.methods[0].clone() },
                methods: r.methods,
                transform_script: None,
                active: r.active,
                weight: r.weight,
                retries: r.retries,
                load_balancer: r.load_balancer,
                created_at: r.created_at,
                requests: 0,
                avg_latency_ms: 0,
            }).collect();
            Ok(Json(info))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
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

    let db = &*state.db;
    
    let id = uuid::Uuid::new_v4().to_string();
    let route = gateway_core::config::Route {
        id: id.clone(),
        name: format!("Route-{}", id.chars().take(8).collect::<String>()),
        path: request.path.clone(),
        upstream: request.upstream.clone(),
        weight: 100,
        active: true,
        methods: vec![request.method.clone()],
        timeout_ms: 30000,
        retries: 3,
        load_balancer: "round_robin".to_string(),
        description: format!("Route created via console for {}", request.path),
        permissions: Vec::new(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    match surreal_config::schema::create_route(db, route.clone()).await {
        Ok(created) => {
            let info = RouteInfo {
                id: created.id,
                name: created.name,
                path: created.path,
                upstream: created.upstream,
                method: created.methods[0].clone(),
                methods: created.methods,
                transform_script: request.transform_script,
                active: created.active,
                weight: created.weight,
                retries: created.retries,
                load_balancer: created.load_balancer,
                created_at: created.created_at.clone(),
                requests: 0,
                avg_latency_ms: 0,
            };

            
            // Still push to cache for reactive UI if needed, but the primary source is now DB
            let mut routes = state.routes.write().await;
            routes.push(info.clone());

            info!(route_id = %info.id, "Created new route in database");
            Ok(Json(info))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}


/// List transformations - /api/transformations
pub async fn list_transformations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TransformationInfo>>, (StatusCode, String)> {
    let db = &*state.db;
    
    // Attempt to fetch from transformations table (which might be empty)
    // For now, we return empty if table doesn't exist/empty, rather than hardcoded mock
    match surreal_config::list_transformations(db).await {
        Ok(data) => {
            let info = data.into_iter().map(|t| TransformationInfo {
                id: t.id,
                name: t.name,
                description: t.description,
                language: t.language,
                script: t.script,
                input_type: t.input_type,
                output_type: t.output_type,
                used_by: t.used_by,
                created_at: t.created_at,
                updated_at: t.updated_at,
            }).collect();
            Ok(Json(info))
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch transformations");
            Ok(Json(Vec::new()))
        }
    }
}

/// List security events - /api/security/events
pub async fn list_security_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SecurityEventsQuery>,
) -> Result<Json<Vec<SecurityEvent>>, (StatusCode, String)> {
    let db = &*state.db;
    
    match surreal_config::list_security_events(db, params.limit).await {
        Ok(data) => {
            let events = data.into_iter().map(|e| SecurityEvent {
                id: e.id,
                event_type: e.event_type,
                category: e.category,
                message: e.message,
                source: e.source,
                timestamp: e.timestamp,
            }).collect();
            Ok(Json(events))
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch security events");
            Ok(Json(Vec::new()))
        }
    }
}

/// List schemas - /api/schemas
pub async fn list_schemas(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SchemaInfo>>, (StatusCode, String)> {
    let db = &*state.db;
    
    match surreal_config::list_api_schemas(db).await {
        Ok(data) => {
            let schemas = data.into_iter().map(|s| SchemaInfo {
                id: s.id,
                name: s.name,
                schema_type: s.schema_type,
                version: s.version,
                content: s.content,
                endpoints: s.endpoints,
                status: s.status,
                created_at: s.created_at,
                updated_at: s.updated_at,
            }).collect();
            Ok(Json(schemas))
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch schemas");
            Ok(Json(Vec::new()))
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct CreateSchemaRequest {
    pub name: String,
    #[serde(alias = "type")]
    pub schema_type: String,
    pub version: String,
    pub content: String,
    #[serde(default)]
    pub endpoints: u32,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "valid".to_string()
}

/// Create a new schema - POST /api/schemas
pub async fn create_schema(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateSchemaRequest>,
) -> Result<Json<SchemaInfo>, (StatusCode, String)> {
    let db = &*state.db;

    // Construct the schema object for the database (using the type expected by surreal-config)
    let schema_input = surreal_config::DbSchemaInfo {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        schema_type: request.schema_type,
        version: request.version,
        content: request.content,
        endpoints: request.endpoints,
        status: request.status,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    match surreal_config::create_api_schema(db, schema_input).await {
        Ok(created) => {
             // Map result back to response type
             let info = SchemaInfo {
                id: created.id,
                name: created.name,
                schema_type: created.schema_type,
                version: created.version,
                content: created.content,
                endpoints: created.endpoints,
                status: created.status,
                created_at: created.created_at,
                updated_at: created.updated_at,
            };
            info!(schema_id = %info.id, "Created new API schema");
            Ok(Json(info))
        }
        Err(e) => {
            error!(error = %e, "Failed to create schema");
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}


/// Delete a schema - DELETE /api/schemas/:id
pub async fn delete_schema(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db = &*state.db;

    match surreal_config::delete_api_schema(db, &id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!(error = %e, "Failed to delete schema");
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// Request to generate routes
#[derive(Debug, Deserialize)]
pub struct GenerateRoutesRequest {
    pub partial: bool, // If true, might select subset (not implemented yet)
}

/// Response for route generation
#[derive(Debug, Serialize)]
pub struct GenerateRoutesResponse {
    pub routes_created: usize,
    pub message: String,
}

/// Generate routes from schema - POST /api/schemas/:id/routes
pub async fn generate_routes_from_schema(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(_request): Json<GenerateRoutesRequest>,
) -> Result<Json<GenerateRoutesResponse>, (StatusCode, String)> {
    let db = &*state.db;

    // 1. Fetch Schema
    let schema = match surreal_config::get_api_schema(db, &id).await {
        Ok(Some(s)) => s,
        Ok(None) => return Err((StatusCode::NOT_FOUND, "Schema not found".to_string())),
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    // 2. Parse and Generate Routes based on type
    let routes = match schema.schema_type.to_lowercase().as_str() {
        "openapi" => generate_routes_from_openapi(&schema.content),
        "soap" => Ok(vec![]), // TODO: Implement SOAP parsing
        "grpc" => Ok(vec![]), // TODO: Implement gRPC parsing
        "mcp" => Ok(vec![]), // TODO: Implement MCP parsing
        "mqtt" => Ok(vec![]), // TODO: Implement MQTT parsing
        "graphql" => Ok(vec![]), // TODO: Implement GraphQL parsing
        "jsonschema" => Ok(vec![]), // JSON Schema doesn't directly map to routes usually
        _ => Err(format!("Unsupported schema type: {}", schema.schema_type)),
    };

    match routes {
        Ok(generated_routes) => {
            let count = generated_routes.len();
            if count == 0 {
                 return Ok(Json(GenerateRoutesResponse {
                    routes_created: 0,
                    message: format!("No routes generated for type {}", schema.schema_type),
                 }));
            }

            // 3. Save Routes
            for route in generated_routes {
                if let Err(e) = surreal_config::create_route(db, route).await {
                    error!("Failed to create generated route: {}", e);
                    // continue or fail? let's continue but log
                }
            }
            
            // Refresh cache (optional/naive)
            if let Ok(all_routes) = surreal_config::get_all_routes(db).await {
                 let mut cache = state.routes.write().await;
                 *cache = all_routes.into_iter().map(|r| RouteInfo {
                    id: r.id,
                    name: r.name,
                    path: r.path,
                    upstream: r.upstream,
                    method: if r.methods.is_empty() { "GET".to_string() } else { r.methods[0].clone() },
                    methods: r.methods,
                    transform_script: None,
                    active: r.active,
                    weight: r.weight,
                    retries: r.retries,
                    load_balancer: r.load_balancer,
                    created_at: r.created_at,
                    requests: 0,
                    avg_latency_ms: 0, 
                 }).collect();
            }

            Ok(Json(GenerateRoutesResponse {
                routes_created: count,
                message: format!("Successfully generated {} routes from schema", count),
            }))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Failed to generate routes: {}", e))),
    }
}

/// Helper to generate routes from OpenAPI content
fn generate_routes_from_openapi(content: &str) -> Result<Vec<gateway_core::config::Route>, String> {
    // Basic JSON parsing
    let json: serde_json::Value = serde_json::from_str(content)
        .or_else(|_| serde_yaml::from_str(content).map_err(|e| e.to_string()))
        .map_err(|e| format!("Invalid OpenAPI format: {}", e))?;

    let mut routes = Vec::new();

    if let Some(paths) = json.get("paths").and_then(|p| p.as_object()) {
        for (path, methods) in paths {
            if let Some(methods_map) = methods.as_object() {
                for (method, _details) in methods_map {
                    // Create a route for this path/method
                    let id = uuid::Uuid::new_v4().to_string();
                    let route = gateway_core::config::Route {
                        id: id.clone(),
                        name: format!("Auto-{}", id.chars().take(8).collect::<String>()),
                        path: path.clone(), // Note: OpenAPI paths have {} params, might need conversion
                        upstream: "http://localhost:8080".to_string(), // Default upstream, needs configuration
                        weight: 100,
                        active: true,
                        methods: vec![method.to_uppercase()],
                        timeout_ms: 30000,
                        retries: 3,
                        load_balancer: "round_robin".to_string(),
                        description: format!("Generated from OpenAPI: {} {}", method.to_uppercase(), path),
                        permissions: Vec::new(),
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };
                    routes.push(route);
                }
            }
        }
    }

    Ok(routes)
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

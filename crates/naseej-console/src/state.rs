//! Application state for the console API server

use cognitive_core::{ArchitectConfig, NaseejArchitect, RhaiEngine, VectorStore};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;

/// Shared application state
pub struct AppState {
    /// AI Architect
    pub architect: RwLock<NaseejArchitect>,

    /// Routes cache (in-memory for demo)
    pub routes: RwLock<Vec<RouteInfo>>,

    /// Transformations cache
    pub transformations: RwLock<Vec<TransformationInfo>>,

    /// Security events log
    pub security_events: RwLock<Vec<SecurityEvent>>,

    /// API Schemas
    pub schemas: RwLock<Vec<SchemaInfo>>,

    /// Persistent Database Connection
    pub db: Arc<Surreal<Client>>,

    /// Server start time for uptime calculation
    pub start_time: Instant,
}

/// Route information for the UI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RouteInfo {
    pub id: String,
    pub path: String,
    pub upstream: String,
    pub method: String,
    pub transform_script: Option<String>,
    pub active: bool,
    pub created_at: String,
    #[serde(default)]
    pub requests: u64,
    #[serde(default)]
    pub avg_latency_ms: u64,
}

/// Transformation script info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransformationInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub script: String,
    pub input_type: String,
    pub output_type: String,
    pub used_by: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Security event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub category: String,
    pub message: String,
    pub source: String,
    pub timestamp: String,
}

/// API Schema info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchemaInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub schema_type: String,
    pub version: String,
    pub content: String,
    pub endpoints: u32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl AppState {
    /// Create new application state with demo data
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        let rhai_engine = Arc::new(RhaiEngine::new());
        let vector_store = Arc::new(RwLock::new(VectorStore::new()));
        let config = ArchitectConfig::default();
        let architect = NaseejArchitect::new(config, rhai_engine, vector_store);

        // Sample routes for demo
        let routes = vec![
            RouteInfo {
                id: "route-1".to_string(),
                path: "/api/users".to_string(),
                upstream: "http://users-service:8080".to_string(),
                method: "GET".to_string(),
                transform_script: None,
                active: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                requests: 12453,
                avg_latency_ms: 42,
            },
            RouteInfo {
                id: "route-2".to_string(),
                path: "/api/orders".to_string(),
                upstream: "http://orders-service:8080".to_string(),
                method: "POST".to_string(),
                transform_script: Some("output = input;".to_string()),
                active: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                requests: 8921,
                avg_latency_ms: 67,
            },
            RouteInfo {
                id: "route-3".to_string(),
                path: "/api/products".to_string(),
                upstream: "http://products-service:8080".to_string(),
                method: "GET".to_string(),
                transform_script: None,
                active: false,
                created_at: chrono::Utc::now().to_rfc3339(),
                requests: 5432,
                avg_latency_ms: 35,
            },
        ];

        // Sample transformations
        let transformations = vec![
            TransformationInfo {
                id: "transform-1".to_string(),
                name: "Celsius to Fahrenheit".to_string(),
                description: "Converts temperature values".to_string(),
                language: "rhai".to_string(),
                script: "let temp = input.temperature;\noutput.fahrenheit = (temp * 9/5) + 32;".to_string(),
                input_type: "json".to_string(),
                output_type: "json".to_string(),
                used_by: vec!["route-1".to_string()],
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
            TransformationInfo {
                id: "transform-2".to_string(),
                name: "XML to JSON".to_string(),
                description: "Converts XML payloads to JSON".to_string(),
                language: "rhai".to_string(),
                script: "output = xml_to_json(input);".to_string(),
                input_type: "xml".to_string(),
                output_type: "json".to_string(),
                used_by: vec!["route-2".to_string()],
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        ];

        // Sample security events
        let security_events = vec![
            SecurityEvent {
                id: "sec-1".to_string(),
                event_type: "blocked".to_string(),
                category: "waf".to_string(),
                message: "SQL Injection attempt detected".to_string(),
                source: "192.168.1.45".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            SecurityEvent {
                id: "sec-2".to_string(),
                event_type: "warning".to_string(),
                category: "rate_limit".to_string(),
                message: "Rate limit exceeded".to_string(),
                source: "10.0.0.23".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            SecurityEvent {
                id: "sec-3".to_string(),
                event_type: "blocked".to_string(),
                category: "waf".to_string(),
                message: "XSS payload in request body".to_string(),
                source: "192.168.1.89".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ];

        // Sample schemas
        let schemas = vec![
            SchemaInfo {
                id: "schema-1".to_string(),
                name: "User Service API".to_string(),
                schema_type: "openapi".to_string(),
                version: "3.0.1".to_string(),
                content: "".to_string(),
                endpoints: 12,
                status: "valid".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
            SchemaInfo {
                id: "schema-2".to_string(),
                name: "Orders Schema".to_string(),
                schema_type: "jsonschema".to_string(),
                version: "draft-07".to_string(),
                content: "".to_string(),
                endpoints: 5,
                status: "valid".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        ];

        Self {
            architect: RwLock::new(architect),
            routes: RwLock::new(routes),
            transformations: RwLock::new(transformations),
            security_events: RwLock::new(security_events),
            schemas: RwLock::new(schemas),
            db,
            start_time: Instant::now(),
        }
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

// impl Default for AppState removal since it requires args now

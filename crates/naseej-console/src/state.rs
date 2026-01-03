//! Application state for the console API server

use cognitive_core::{ArchitectConfig, NaseejArchitect, RhaiEngine, VectorStore};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use surrealdb::Surreal;
use surreal_config::db::RemoteDb;
use surreal_config::ConfigError;
use tracing::info;

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
    pub db: Arc<RemoteDb>,

    /// Server start time for uptime calculation
    pub start_time: Instant,
}

/// Route information for the UI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RouteInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub upstream: String,
    pub method: String, // UI currently expects single method or primary
    pub methods: Vec<String>,
    pub transform_script: Option<String>,
    pub active: bool,
    pub weight: u32,
    pub retries: u32,
    pub load_balancer: String,
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
    /// Create new application state
    pub fn new(db: Arc<RemoteDb>) -> Self {
        let rhai_engine = Arc::new(RhaiEngine::new());
        let vector_store = Arc::new(RwLock::new(VectorStore::new()));
        let config = ArchitectConfig::default();
        let architect = NaseejArchitect::new(config, rhai_engine, vector_store);

        Self {
            architect: RwLock::new(architect),
            routes: RwLock::new(Vec::new()),
            transformations: RwLock::new(Vec::new()),
            security_events: RwLock::new(Vec::new()),
            schemas: RwLock::new(Vec::new()),
            db,
            start_time: Instant::now(),
        }
    }

    /// Load state from database
    pub async fn load_from_db(&self) -> Result<(), surreal_config::ConfigError> {
        let db = &*self.db;

        // Fetch routes
        let routes = surreal_config::schema::get_all_routes(db).await?;
        let mut routes_cache = self.routes.write().await;
        *routes_cache = routes.into_iter().map(|r| RouteInfo {
            id: r.id.clone(),
            name: r.name,
            path: r.path.clone(),
            upstream: r.upstream.clone(),
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

        // Fetch transformations
        if let Ok(transformations) = surreal_config::list_transformations(db).await {
            let mut trans_cache = self.transformations.write().await;
            *trans_cache = transformations.into_iter().map(|t| TransformationInfo {
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
        }

        // Fetch schemas
        if let Ok(schemas) = surreal_config::list_api_schemas(db).await {
            let mut schema_cache = self.schemas.write().await;
            *schema_cache = schemas.into_iter().map(|s| SchemaInfo {
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
        }

        // Fetch security events
        if let Ok(events) = surreal_config::list_security_events(db, 50).await {
            let mut event_cache = self.security_events.write().await;
            *event_cache = events.into_iter().map(|e| SecurityEvent {
                id: e.id,
                event_type: e.event_type,
                category: e.category,
                message: e.message,
                source: e.source,
                timestamp: e.timestamp,
            }).collect();
        }

        info!(
            routes = %routes_cache.len(),
            transformations = %self.transformations.read().await.len(),
            schemas = %self.schemas.read().await.len(),
            "Loaded mesh state from database"
        );


        Ok(())
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}


// impl Default for AppState removal since it requires args now

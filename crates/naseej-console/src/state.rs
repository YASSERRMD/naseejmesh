//! Application state for the console API server

use cognitive_core::{ArchitectConfig, NaseejArchitect, RhaiEngine, VectorStore};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state
pub struct AppState {
    /// AI Architect
    pub architect: RwLock<NaseejArchitect>,

    /// Routes cache (in-memory for demo)
    pub routes: RwLock<Vec<RouteInfo>>,
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
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        let rhai_engine = Arc::new(RhaiEngine::new());
        let vector_store = Arc::new(RwLock::new(VectorStore::new()));
        let config = ArchitectConfig::default();
        let architect = NaseejArchitect::new(config, rhai_engine, vector_store);

        Self {
            architect: RwLock::new(architect),
            routes: RwLock::new(Vec::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

//! Vector Schema for SurrealDB
//!
//! Defines the schema and types for storing API endpoint vectors in SurrealDB.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A vector record stored in SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    /// Endpoint identifier
    pub endpoint_id: String,
    
    /// The text that was embedded
    pub text: String,
    
    /// Embedding vector (1024 dimensions for Cohere embed-english-v3.0)
    pub embedding: Vec<f32>,
    
    /// HTTP method
    pub method: String,
    
    /// API path
    pub path: String,
    
    /// Summary/description
    pub summary: Option<String>,
    
    /// Source specification file
    pub source_spec: String,
    
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Initialize the vector schema in SurrealDB
pub async fn init_vector_schema<C: surrealdb::Connection>(
    db: &surrealdb::Surreal<C>,
) -> Result<(), surrealdb::Error> {
    tracing::info!("Initializing api_vectors schema...");
    
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS api_vectors SCHEMAFULL;
        DEFINE FIELD endpoint_id ON api_vectors TYPE string;
        DEFINE FIELD text ON api_vectors TYPE string;
        DEFINE FIELD embedding ON api_vectors TYPE array;
        DEFINE FIELD method ON api_vectors TYPE string;
        DEFINE FIELD path ON api_vectors TYPE string;
        DEFINE FIELD summary ON api_vectors TYPE option<string>;
        DEFINE FIELD source_spec ON api_vectors TYPE string;
        DEFINE FIELD metadata ON api_vectors TYPE object DEFAULT {};
        DEFINE FIELD created_at ON api_vectors TYPE datetime;
        DEFINE INDEX endpoint_idx ON api_vectors FIELDS endpoint_id UNIQUE;
        DEFINE INDEX source_idx ON api_vectors FIELDS source_spec;
        ",
    )
    .await?;
    
    tracing::info!("api_vectors schema initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_record_serialization() {
        let record = VectorRecord {
            endpoint_id: "test-1".to_string(),
            text: "GET /users - List all users".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            method: "GET".to_string(),
            path: "/users".to_string(),
            summary: Some("List all users".to_string()),
            source_spec: "openapi.yaml".to_string(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("endpoint_id"));
        assert!(json.contains("embedding"));
    }
}

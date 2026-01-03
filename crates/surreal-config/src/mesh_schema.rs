//! Mesh schema and CRUD operations for Transformations, Schemas, and Security Events.

use serde::{Deserialize, Serialize};
use surrealdb::Connection;
use surrealdb::Surreal;
use crate::error::ConfigError;

// ============================================================================
// Internal Database Structures (SurrealDB v2 compatible)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct DbTransformation {
    id: surrealdb::sql::Thing,
    name: String,
    description: String,
    language: String,
    script: String,
    input_type: String,
    output_type: String,
    used_by: Vec<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DbApiSchema {
    id: surrealdb::sql::Thing,
    name: String,
    #[serde(rename = "type")]
    schema_type: String, // mapped from "type" in SQL
    version: String,
    content: String,
    endpoints: u32,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DbSecurityEvent {
    id: surrealdb::sql::Thing,
    #[serde(rename = "type")]
    event_type: String, // mapped from "type" in SQL
    category: String,
    message: String,
    source: String,
    timestamp: String,
}

// ============================================================================
// Public Info Structures (Matching naseej-console types)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub id: String,
    pub name: String,
    pub schema_type: String,
    pub version: String,
    pub content: String,
    pub endpoints: u32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEventInfo {
    pub id: String,
    pub event_type: String,
    pub category: String,
    pub message: String,
    pub source: String,
    pub timestamp: String,
}

// ============================================================================
// Mappings
// ============================================================================

impl From<DbTransformation> for TransformationInfo {
    fn from(db: DbTransformation) -> Self {
        Self {
            id: db.id.to_string(),
            name: db.name,
            description: db.description,
            language: db.language,
            script: db.script,
            input_type: db.input_type,
            output_type: db.output_type,
            used_by: db.used_by,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

impl From<DbApiSchema> for SchemaInfo {
    fn from(db: DbApiSchema) -> Self {
        Self {
            id: db.id.to_string(),
            name: db.name,
            schema_type: db.schema_type,
            version: db.version,
            content: db.content,
            endpoints: db.endpoints,
            status: db.status,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

impl From<DbSecurityEvent> for SecurityEventInfo {
    fn from(db: DbSecurityEvent) -> Self {
        Self {
            id: db.id.to_string(),
            event_type: db.event_type,
            category: db.category,
            message: db.message,
            source: db.source,
            timestamp: db.timestamp,
        }
    }
}

// ============================================================================
// CRUD Operations
// ============================================================================

pub async fn list_transformations<C: Connection>(db: &Surreal<C>) -> Result<Vec<TransformationInfo>, ConfigError> {
    let result: Vec<DbTransformation> = db.select("transformations").await?;
    Ok(result.into_iter().map(TransformationInfo::from).collect())
}

pub async fn list_api_schemas<C: Connection>(db: &Surreal<C>) -> Result<Vec<SchemaInfo>, ConfigError> {
    let result: Vec<DbApiSchema> = db.select("api_schemas").await?;
    Ok(result.into_iter().map(SchemaInfo::from).collect())
}

pub async fn list_security_events<C: Connection>(db: &Surreal<C>, limit: usize) -> Result<Vec<SecurityEventInfo>, ConfigError> {
    let mut result = db.query("SELECT * FROM security_events ORDER BY timestamp DESC LIMIT $limit")
        .bind(("limit", limit))
        .await?;
    let events: Vec<DbSecurityEvent> = result.take(0)?;
    Ok(events.into_iter().map(SecurityEventInfo::from).collect())
}

pub async fn create_security_event<C: Connection>(db: &Surreal<C>, event: SecurityEventInfo) -> Result<SecurityEventInfo, ConfigError> {
    let mut result = db.query("CREATE security_events SET type = $type, category = $cat, message = $msg, source = $src, timestamp = $ts")
        .bind(("type", event.event_type))
        .bind(("cat", event.category))
        .bind(("msg", event.message))
        .bind(("src", event.source))
        .bind(("ts", event.timestamp))
        .await?;
    let created: Option<DbSecurityEvent> = result.take(0)?;
    created.map(SecurityEventInfo::from).ok_or_else(|| ConfigError::Database("Failed to create security event".to_string()))
}

pub async fn create_api_schema<C: Connection>(db: &Surreal<C>, schema: SchemaInfo) -> Result<SchemaInfo, ConfigError> {
    let mut result = db.query("CREATE api_schemas SET name = $name, type = $type, version = $version, content = $content, endpoints = $endpoints, status = $status, created_at = $created_at, updated_at = $updated_at")
        .bind(("name", schema.name))
        .bind(("type", schema.schema_type))
        .bind(("version", schema.version))
        .bind(("content", schema.content))
        .bind(("endpoints", schema.endpoints))
        .bind(("status", schema.status))
        .bind(("created_at", schema.created_at))
        .bind(("updated_at", schema.updated_at))
        .await?;
    let created: Option<DbApiSchema> = result.take(0)?;
    created.map(SchemaInfo::from).ok_or_else(|| ConfigError::Database("Failed to create schema".to_string()))
}

pub async fn delete_api_schema<C: Connection>(db: &Surreal<C>, id: &str) -> Result<(), ConfigError> {
    // Handle "api_schemas:id" vs "id"
    let id_part = id.split(':').last().unwrap_or(id);
    let _result: Option<DbApiSchema> = db.delete(("api_schemas", id_part)).await?;
    Ok(())
}


pub async fn get_api_schema<C: Connection>(db: &Surreal<C>, id: &str) -> Result<Option<SchemaInfo>, ConfigError> {
    // Handle "api_schemas:id" vs "id"
    let id_part = id.split(':').last().unwrap_or(id);
    let result: Option<DbApiSchema> = db.select(("api_schemas", id_part)).await?;
    Ok(result.map(SchemaInfo::from))
}

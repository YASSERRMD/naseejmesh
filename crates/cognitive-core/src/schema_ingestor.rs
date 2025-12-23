//! Schema Ingestion for RAG Pipeline
//!
//! Parses OpenAPI specifications and extracts endpoints for vectorization.
//! This enables the AI to "know" about available APIs.

use openapiv3::{OpenAPI, Operation, PathItem, ReferenceOr};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};

/// Schema ingestion errors
#[derive(Debug, Error)]
pub enum IngestError {
    #[error("Failed to parse OpenAPI spec: {0}")]
    ParseError(String),

    #[error("Invalid spec format: {0}")]
    InvalidFormat(String),

    #[error("Embedding failed: {0}")]
    EmbeddingError(String),

    #[error("Storage failed: {0}")]
    StorageError(String),
}

/// Represents an API endpoint extracted from a spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    /// Unique identifier
    pub id: String,

    /// HTTP method (GET, POST, etc.)
    pub method: String,

    /// Path template (e.g., /users/{id})
    pub path: String,

    /// Operation summary
    pub summary: Option<String>,

    /// Operation description
    pub description: Option<String>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Request body content type
    pub request_content_type: Option<String>,

    /// Response content type
    pub response_content_type: Option<String>,

    /// Parameters (path, query, header)
    pub parameters: Vec<ParameterInfo>,

    /// Combined text for embedding
    pub embedding_text: String,

    /// Source spec identifier
    pub source_spec: String,
}

/// Parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub location: String, // path, query, header
    pub required: bool,
    pub description: Option<String>,
}

/// Schema ingestor for parsing API specifications
pub struct SchemaIngestor {
    /// Source identifier for tracking
    source_id: String,
}

impl SchemaIngestor {
    /// Create a new ingestor with source identifier
    pub fn new(source_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
        }
    }

    /// Parse an OpenAPI specification from YAML or JSON
    pub fn parse_openapi(&self, content: &str) -> Result<Vec<ApiEndpoint>, IngestError> {
        // Try YAML first, then JSON
        let spec: OpenAPI = serde_yaml::from_str(content)
            .or_else(|_| serde_json::from_str(content))
            .map_err(|e| IngestError::ParseError(e.to_string()))?;

        info!(
            title = %spec.info.title,
            version = %spec.info.version,
            "Parsing OpenAPI specification"
        );

        let mut endpoints = Vec::new();

        for (path, path_item) in &spec.paths.paths {
            let path_item = match path_item {
                ReferenceOr::Item(item) => item,
                ReferenceOr::Reference { .. } => continue,
            };

            endpoints.extend(self.extract_operations(path, path_item));
        }

        info!(count = endpoints.len(), "Extracted API endpoints");
        Ok(endpoints)
    }

    /// Extract operations from a path item
    fn extract_operations(&self, path: &str, item: &PathItem) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        let methods = [
            ("GET", &item.get),
            ("POST", &item.post),
            ("PUT", &item.put),
            ("DELETE", &item.delete),
            ("PATCH", &item.patch),
            ("HEAD", &item.head),
            ("OPTIONS", &item.options),
        ];

        for (method, operation) in methods {
            if let Some(op) = operation {
                endpoints.push(self.create_endpoint(path, method, op));
            }
        }

        endpoints
    }

    /// Create an endpoint from an operation
    fn create_endpoint(&self, path: &str, method: &str, op: &Operation) -> ApiEndpoint {
        let id = format!(
            "{}-{}-{}",
            self.source_id,
            method.to_lowercase(),
            path.replace('/', "-").trim_matches('-')
        );

        let parameters: Vec<ParameterInfo> = op
            .parameters
            .iter()
            .filter_map(|p| {
                match p {
                    ReferenceOr::Item(param) => {
                        let (name, location, required, description) = match param {
                            openapiv3::Parameter::Query { parameter_data, .. } => {
                                (
                                    parameter_data.name.clone(),
                                    "query".to_string(),
                                    parameter_data.required,
                                    parameter_data.description.clone(),
                                )
                            }
                            openapiv3::Parameter::Header { parameter_data, .. } => {
                                (
                                    parameter_data.name.clone(),
                                    "header".to_string(),
                                    parameter_data.required,
                                    parameter_data.description.clone(),
                                )
                            }
                            openapiv3::Parameter::Path { parameter_data, .. } => {
                                (
                                    parameter_data.name.clone(),
                                    "path".to_string(),
                                    parameter_data.required,
                                    parameter_data.description.clone(),
                                )
                            }
                            openapiv3::Parameter::Cookie { parameter_data, .. } => {
                                (
                                    parameter_data.name.clone(),
                                    "cookie".to_string(),
                                    parameter_data.required,
                                    parameter_data.description.clone(),
                                )
                            }
                        };
                        Some(ParameterInfo {
                            name,
                            location,
                            required,
                            description,
                        })
                    }
                    _ => None,
                }
            })
            .collect();

        // Build embedding text
        let mut embedding_parts = vec![
            format!("{} {}", method, path),
        ];

        if let Some(summary) = &op.summary {
            embedding_parts.push(summary.clone());
        }

        if let Some(desc) = &op.description {
            embedding_parts.push(desc.clone());
        }

        for tag in &op.tags {
            embedding_parts.push(tag.clone());
        }

        for param in &parameters {
            embedding_parts.push(format!("parameter {} ({})", param.name, param.location));
        }

        let embedding_text = embedding_parts.join(" | ");

        debug!(
            id = %id,
            method = %method,
            path = %path,
            "Created API endpoint"
        );

        ApiEndpoint {
            id,
            method: method.to_string(),
            path: path.to_string(),
            summary: op.summary.clone(),
            description: op.description.clone(),
            tags: op.tags.clone(),
            request_content_type: op.request_body.as_ref().and_then(|rb| {
                match rb {
                    ReferenceOr::Item(body) => body.content.keys().next().cloned(),
                    _ => None,
                }
            }),
            response_content_type: op.responses.default.as_ref().and_then(|r| {
                match r {
                    ReferenceOr::Item(resp) => resp.content.keys().next().cloned(),
                    _ => None,
                }
            }),
            parameters,
            embedding_text,
            source_spec: self.source_id.clone(),
        }
    }

    /// Generate a searchable text summary of an endpoint
    pub fn endpoint_to_search_text(endpoint: &ApiEndpoint) -> String {
        endpoint.embedding_text.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OPENAPI_JSON: &str = r#"{
        "openapi": "3.0.0",
        "info": {
            "title": "Sample API",
            "version": "1.0.0"
        },
        "paths": {
            "/users": {
                "get": {
                    "summary": "List all users",
                    "tags": ["users"],
                    "responses": {
                        "200": {"description": "Success"}
                    }
                },
                "post": {
                    "summary": "Create a user",
                    "tags": ["users"],
                    "responses": {
                        "201": {"description": "Created"}
                    }
                }
            },
            "/users/{id}": {
                "get": {
                    "summary": "Get user by ID",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": {"type": "string"}
                        }
                    ],
                    "responses": {
                        "200": {"description": "Success"}
                    }
                }
            }
        }
    }"#;

    #[test]
    fn test_parse_openapi() {
        let ingestor = SchemaIngestor::new("sample-api");
        let endpoints = ingestor.parse_openapi(SAMPLE_OPENAPI_JSON).unwrap();

        assert_eq!(endpoints.len(), 3);
        
        let get_users = endpoints.iter().find(|e| e.method == "GET" && e.path == "/users");
        assert!(get_users.is_some());
        assert_eq!(get_users.unwrap().summary, Some("List all users".to_string()));
    }

    #[test]
    fn test_endpoint_parameters() {
        let ingestor = SchemaIngestor::new("sample-api");
        let endpoints = ingestor.parse_openapi(SAMPLE_OPENAPI_JSON).unwrap();

        let get_user = endpoints.iter().find(|e| e.path == "/users/{id}").unwrap();
        assert_eq!(get_user.parameters.len(), 1);
        assert_eq!(get_user.parameters[0].name, "id");
        assert_eq!(get_user.parameters[0].location, "path");
        assert!(get_user.parameters[0].required);
    }

    #[test]
    fn test_embedding_text() {
        let ingestor = SchemaIngestor::new("sample-api");
        let endpoints = ingestor.parse_openapi(SAMPLE_OPENAPI_JSON).unwrap();

        let get_users = endpoints.iter().find(|e| e.method == "GET" && e.path == "/users").unwrap();
        assert!(get_users.embedding_text.contains("GET /users"));
        assert!(get_users.embedding_text.contains("List all users"));
    }
}

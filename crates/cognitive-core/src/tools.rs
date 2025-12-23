//! AI Tools for the Architect
//!
//! Defines the tools that the AI agent can use to interact with the gateway.
//! These tools give the AI "hands" to deploy routes, lookup schemas, etc.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::rhai_engine::RhaiEngine;
use crate::vector_store::VectorStore;

/// Tool execution errors
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

/// Result of a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the call succeeded
    pub success: bool,

    /// Result message
    pub message: String,

    /// Additional data (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ToolResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }

    pub fn success_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }
}

/// Arguments for deploying a route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployRouteArgs {
    /// Route path (e.g., "/api/users")
    pub path: String,

    /// Upstream URL
    pub upstream: String,

    /// Methods to allow (GET, POST, etc.)
    #[serde(default)]
    pub methods: Vec<String>,

    /// Rhai transformation script (optional)
    #[serde(default)]
    pub transform_script: Option<String>,

    /// Route description
    #[serde(default)]
    pub description: Option<String>,
}

/// Arguments for looking up a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupSchemaArgs {
    /// Search query
    pub query: String,

    /// Max results to return
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    5
}

/// Arguments for validating a Rhai script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRhaiArgs {
    /// The script to validate
    pub script: String,

    /// Optional test payload
    #[serde(default)]
    pub test_payload: Option<serde_json::Value>,
}

/// Tool for deploying routes
pub struct DeployRouteTool {
    /// Rhai engine for script validation
    rhai_engine: Arc<RhaiEngine>,
}

impl DeployRouteTool {
    pub fn new(rhai_engine: Arc<RhaiEngine>) -> Self {
        Self { rhai_engine }
    }

    /// Execute the deploy route tool
    pub async fn call(&self, args: DeployRouteArgs) -> ToolResult {
        info!(
            path = %args.path,
            upstream = %args.upstream,
            "Deploying route"
        );

        // Validate the path
        if !args.path.starts_with('/') {
            return ToolResult::error("Path must start with /");
        }

        // Validate upstream URL
        if !args.upstream.starts_with("http://") && !args.upstream.starts_with("https://") {
            return ToolResult::error("Upstream must be a valid HTTP(S) URL");
        }

        // Validate Rhai script if provided
        if let Some(script) = &args.transform_script {
            let validation = self.rhai_engine.validate(script);
            if !validation.valid {
                return ToolResult::error(format!(
                    "Transform script validation failed: {}",
                    validation.errors.join(", ")
                ));
            }
        }

        // In production, this would write to SurrealDB
        // For now, we simulate success
        let route_id = format!(
            "route-{}",
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
        );

        debug!(route_id = %route_id, "Route created");

        ToolResult::success_with_data(
            format!("Route deployed successfully. ID: {}", route_id),
            serde_json::json!({
                "route_id": route_id,
                "path": args.path,
                "upstream": args.upstream,
                "status": "active"
            }),
        )
    }
}

/// Tool for looking up API schemas
pub struct LookupSchemaTool {
    /// Vector store for semantic search
    vector_store: Arc<RwLock<VectorStore>>,
}

impl LookupSchemaTool {
    pub fn new(vector_store: Arc<RwLock<VectorStore>>) -> Self {
        Self { vector_store }
    }

    /// Execute the lookup schema tool
    pub async fn call(&self, args: LookupSchemaArgs) -> ToolResult {
        info!(query = %args.query, "Looking up schema");

        let store = self.vector_store.read().await;
        
        match store.search(&args.query, args.limit).await {
            Ok(results) if results.is_empty() => {
                ToolResult::error("No matching schemas found")
            }
            Ok(results) => {
                let data: Vec<serde_json::Value> = results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "endpoint_id": r.endpoint_id,
                            "text": r.text,
                            "score": r.score,
                            "metadata": r.metadata
                        })
                    })
                    .collect();

                ToolResult::success_with_data(
                    format!("Found {} matching schemas", results.len()),
                    serde_json::Value::Array(data),
                )
            }
            Err(e) => ToolResult::error(format!("Search failed: {}", e)),
        }
    }
}

/// Tool for validating Rhai scripts
pub struct ValidateRhaiTool {
    /// Rhai engine
    rhai_engine: Arc<RhaiEngine>,
}

impl ValidateRhaiTool {
    pub fn new(rhai_engine: Arc<RhaiEngine>) -> Self {
        Self { rhai_engine }
    }

    /// Execute the validate rhai tool
    pub async fn call(&self, args: ValidateRhaiArgs) -> ToolResult {
        info!("Validating Rhai script");

        let validation = self.rhai_engine.validate(&args.script);

        if !validation.valid {
            return ToolResult::error(format!(
                "Script validation failed: {}",
                validation.errors.join(", ")
            ));
        }

        // If test payload provided, try to execute
        if let Some(payload) = args.test_payload {
            let ctx = crate::rhai_engine::TransformContext {
                payload,
                metadata: std::collections::HashMap::new(),
                protocol: "test".to_string(),
                destination: "/test".to_string(),
            };

            match self.rhai_engine.execute(&args.script, &ctx) {
                Ok(result) => {
                    ToolResult::success_with_data(
                        "Script validated and executed successfully",
                        serde_json::json!({
                            "valid": true,
                            "output": result.payload,
                            "warnings": validation.warnings
                        }),
                    )
                }
                Err(e) => ToolResult::error(format!("Script execution failed: {}", e)),
            }
        } else {
            ToolResult::success_with_data(
                "Script validated successfully",
                serde_json::json!({
                    "valid": true,
                    "warnings": validation.warnings
                }),
            )
        }
    }
}

/// Tool registry for the architect
pub struct ToolRegistry {
    pub deploy_route: DeployRouteTool,
    pub lookup_schema: LookupSchemaTool,
    pub validate_rhai: ValidateRhaiTool,
}

impl ToolRegistry {
    pub fn new(
        rhai_engine: Arc<RhaiEngine>,
        vector_store: Arc<RwLock<VectorStore>>,
    ) -> Self {
        Self {
            deploy_route: DeployRouteTool::new(rhai_engine.clone()),
            lookup_schema: LookupSchemaTool::new(vector_store),
            validate_rhai: ValidateRhaiTool::new(rhai_engine),
        }
    }

    /// Get descriptions of all available tools (for AI prompting)
    pub fn tool_descriptions() -> Vec<ToolDescription> {
        vec![
            ToolDescription {
                name: "deploy_route".to_string(),
                description: "Deploy a new integration route to the gateway. Creates a mapping from a path to an upstream URL with optional transformation.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "The route path, e.g., /api/users"},
                        "upstream": {"type": "string", "description": "The upstream URL to forward to"},
                        "methods": {"type": "array", "items": {"type": "string"}, "description": "Allowed HTTP methods"},
                        "transform_script": {"type": "string", "description": "Optional Rhai script for data transformation"},
                        "description": {"type": "string", "description": "Human-readable description"}
                    },
                    "required": ["path", "upstream"]
                }),
            },
            ToolDescription {
                name: "lookup_schema".to_string(),
                description: "Search for API schemas in the knowledge base using semantic search.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query for finding relevant APIs"},
                        "limit": {"type": "integer", "description": "Maximum results to return (default: 5)"}
                    },
                    "required": ["query"]
                }),
            },
            ToolDescription {
                name: "validate_rhai".to_string(),
                description: "Validate a Rhai transformation script and optionally test it with sample data.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "script": {"type": "string", "description": "The Rhai script to validate"},
                        "test_payload": {"type": "object", "description": "Optional test payload to run the script against"}
                    },
                    "required": ["script"]
                }),
            },
        ]
    }
}

/// Description of a tool for AI prompting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deploy_route_validation() {
        let rhai_engine = Arc::new(RhaiEngine::new());
        let tool = DeployRouteTool::new(rhai_engine);

        // Valid route
        let result = tool.call(DeployRouteArgs {
            path: "/api/test".to_string(),
            upstream: "http://localhost:8080".to_string(),
            methods: vec!["GET".to_string()],
            transform_script: None,
            description: None,
        }).await;

        assert!(result.success);

        // Invalid path
        let result = tool.call(DeployRouteArgs {
            path: "no-slash".to_string(),
            upstream: "http://localhost:8080".to_string(),
            methods: vec![],
            transform_script: None,
            description: None,
        }).await;

        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_validate_rhai_tool() {
        let rhai_engine = Arc::new(RhaiEngine::new());
        let tool = ValidateRhaiTool::new(rhai_engine);

        // Valid script
        let result = tool.call(ValidateRhaiArgs {
            script: "let x = 1 + 2;".to_string(),
            test_payload: None,
        }).await;

        assert!(result.success);

        // Invalid script
        let result = tool.call(ValidateRhaiArgs {
            script: "let x = ;".to_string(),
            test_payload: None,
        }).await;

        assert!(!result.success);
    }

    #[test]
    fn test_tool_descriptions() {
        let descriptions = ToolRegistry::tool_descriptions();
        assert_eq!(descriptions.len(), 3);
        assert!(descriptions.iter().any(|d| d.name == "deploy_route"));
        assert!(descriptions.iter().any(|d| d.name == "lookup_schema"));
        assert!(descriptions.iter().any(|d| d.name == "validate_rhai"));
    }
}

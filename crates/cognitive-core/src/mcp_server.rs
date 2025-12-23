//! MCP Protocol Server
//!
//! Implements the Model Context Protocol for external AI tools.
//! Allows Claude Desktop, IDEs, and other clients to interact with the Architect.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};

use crate::architect::NaseejArchitect;
use crate::rhai_engine::RhaiEngine;
use crate::vector_store::VectorStore;


/// MCP Server errors
#[derive(Debug, Error)]
pub enum McpError {
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// MCP Server implementation
pub struct McpServer {
    /// The AI architect instance
    architect: Arc<RwLock<NaseejArchitect>>,

    /// Server info
    info: ServerInfo,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: "NaseejMesh Architect".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "AI-driven integration architect for NaseejMesh gateway".to_string(),
        }
    }
}

/// JSON-RPC Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP Error Codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// MCP Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub arguments: Vec<McpPromptArgument>,
}

/// MCP Prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(
        rhai_engine: Arc<RhaiEngine>,
        vector_store: Arc<RwLock<VectorStore>>,
    ) -> Self {
        let config = crate::architect::ArchitectConfig::default();
        let architect = NaseejArchitect::new(config, rhai_engine, vector_store);

        Self {
            architect: Arc::new(RwLock::new(architect)),
            info: ServerInfo::default(),
        }
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        info!(method = %request.method, "Handling MCP request");

        let result = match request.method.as_str() {
            // MCP Protocol methods
            "initialize" => self.handle_initialize(&request.params).await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_call_tool(&request.params).await,
            "prompts/list" => self.handle_list_prompts().await,
            "prompts/get" => self.handle_get_prompt(&request.params).await,
            
            // Custom methods
            "architect/chat" => self.handle_architect_chat(&request.params).await,
            "architect/clear" => self.handle_architect_clear().await,

            _ => Err(McpError::MethodNotFound(request.method.clone())),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(self.error_to_jsonrpc(&e)),
            },
        }
    }

    /// Handle initialize request
    async fn handle_initialize(
        &self,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        Ok(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "prompts": {},
                "resources": {}
            },
            "serverInfo": self.info
        }))
    }

    /// Handle list tools request
    async fn handle_list_tools(&self) -> Result<serde_json::Value, McpError> {
        let tool_descriptions = crate::tools::ToolRegistry::tool_descriptions();
        
        let tools: Vec<McpTool> = tool_descriptions
            .into_iter()
            .map(|td| McpTool {
                name: td.name,
                description: td.description,
                input_schema: td.parameters,
            })
            .collect();

        Ok(serde_json::json!({
            "tools": tools
        }))
    }

    /// Handle call tool request
    async fn handle_call_tool(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let tool_name = params.get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| McpError::InvalidParams("Missing 'name' parameter".to_string()))?;

        let arguments = params.get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        debug!(tool = %tool_name, "Calling tool");

        // Route to the architect
        let mut architect = self.architect.write().await;
        let prompt = format!(
            "Execute the {} tool with these parameters: {}",
            tool_name,
            serde_json::to_string(&arguments).unwrap_or_default()
        );

        let response = architect.chat(&prompt).await
            .map_err(|e| McpError::ExecutionError(e.to_string()))?;

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": response
            }]
        }))
    }

    /// Handle list prompts request
    async fn handle_list_prompts(&self) -> Result<serde_json::Value, McpError> {
        let prompts = vec![
            McpPrompt {
                name: "create-integration".to_string(),
                description: "Create a new integration route".to_string(),
                arguments: vec![
                    McpPromptArgument {
                        name: "description".to_string(),
                        description: "Natural language description of the integration".to_string(),
                        required: true,
                    },
                ],
            },
            McpPrompt {
                name: "search-apis".to_string(),
                description: "Search for available APIs in the knowledge base".to_string(),
                arguments: vec![
                    McpPromptArgument {
                        name: "query".to_string(),
                        description: "Search query".to_string(),
                        required: true,
                    },
                ],
            },
            McpPrompt {
                name: "validate-transform".to_string(),
                description: "Validate a Rhai transformation script".to_string(),
                arguments: vec![
                    McpPromptArgument {
                        name: "script".to_string(),
                        description: "The Rhai script to validate".to_string(),
                        required: true,
                    },
                ],
            },
        ];

        Ok(serde_json::json!({
            "prompts": prompts
        }))
    }

    /// Handle get prompt request
    async fn handle_get_prompt(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let prompt_name = params.get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| McpError::InvalidParams("Missing 'name' parameter".to_string()))?;

        let arguments = params.get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let prompt_text = match prompt_name {
            "create-integration" => {
                let desc = arguments.get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("Create an integration");
                format!("Create an integration route: {}", desc)
            }
            "search-apis" => {
                let query = arguments.get("query")
                    .and_then(|q| q.as_str())
                    .unwrap_or("search");
                format!("Search for APIs related to: {}", query)
            }
            "validate-transform" => {
                let script = arguments.get("script")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                format!("Validate this Rhai script:\n```rhai\n{}\n```", script)
            }
            _ => return Err(McpError::MethodNotFound(format!("Prompt not found: {}", prompt_name))),
        };

        Ok(serde_json::json!({
            "messages": [{
                "role": "user",
                "content": {
                    "type": "text",
                    "text": prompt_text
                }
            }]
        }))
    }

    /// Handle architect chat request
    async fn handle_architect_chat(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let message = params.get("message")
            .and_then(|m| m.as_str())
            .ok_or_else(|| McpError::InvalidParams("Missing 'message' parameter".to_string()))?;

        let mut architect = self.architect.write().await;
        let response = architect.chat(message).await
            .map_err(|e| McpError::ExecutionError(e.to_string()))?;

        Ok(serde_json::json!({
            "response": response
        }))
    }

    /// Handle architect clear request
    async fn handle_architect_clear(&self) -> Result<serde_json::Value, McpError> {
        let mut architect = self.architect.write().await;
        architect.clear_history();

        Ok(serde_json::json!({
            "success": true,
            "message": "Conversation history cleared"
        }))
    }

    /// Convert error to JSON-RPC error
    fn error_to_jsonrpc(&self, error: &McpError) -> JsonRpcError {
        let code = match error {
            McpError::MethodNotFound(_) => error_codes::METHOD_NOT_FOUND,
            McpError::InvalidParams(_) => error_codes::INVALID_PARAMS,
            McpError::ExecutionError(_) => error_codes::INTERNAL_ERROR,
            McpError::SerializationError(_) => error_codes::PARSE_ERROR,
        };

        JsonRpcError {
            code,
            message: error.to_string(),
            data: None,
        }
    }

    /// Get server info
    pub fn info(&self) -> &ServerInfo {
        &self.info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rhai_engine::RhaiEngine;
    use crate::vector_store::VectorStore;

    fn create_server() -> McpServer {
        let rhai_engine = Arc::new(RhaiEngine::new());
        let vector_store = Arc::new(RwLock::new(VectorStore::new()));
        McpServer::new(rhai_engine, vector_store)
    }

    #[tokio::test]
    async fn test_initialize() {
        let server = create_server();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "initialize".to_string(),
            params: serde_json::json!({}),
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_list_tools() {
        let server = create_server();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(2),
            method: "tools/list".to_string(),
            params: serde_json::json!({}),
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        
        let result = response.result.unwrap();
        assert!(result.get("tools").is_some());
    }

    #[tokio::test]
    async fn test_list_prompts() {
        let server = create_server();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(3),
            method: "prompts/list".to_string(),
            params: serde_json::json!({}),
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        
        let result = response.result.unwrap();
        assert!(result.get("prompts").is_some());
    }

    #[tokio::test]
    async fn test_method_not_found() {
        let server = create_server();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(4),
            method: "unknown/method".to_string(),
            params: serde_json::json!({}),
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, error_codes::METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn test_architect_chat() {
        let server = create_server();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(5),
            method: "architect/chat".to_string(),
            params: serde_json::json!({
                "message": "Hello, what can you do?"
            }),
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
    }
}

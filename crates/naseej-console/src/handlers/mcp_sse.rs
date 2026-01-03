//! MCP SSE Streaming Handler
//!
//! Server-Sent Events handler for real-time AI chat streaming.

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use cognitive_core::AiArchitect;

/// Chat request payload
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    /// User message
    pub message: String,
    /// Optional session ID for multi-turn conversations
    pub session_id: Option<String>,
}

/// SSE message types
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum SseMessage {
    #[serde(rename = "thinking")]
    Thinking { content: String },
    
    #[serde(rename = "content")]
    Content { content: String },
    
    #[serde(rename = "tool_call")]
    ToolCall { 
        name: String, 
        args: serde_json::Value,
    },
    
    #[serde(rename = "tool_result")]
    ToolResult {
        name: String,
        success: bool,
        message: String,
    },
    
    #[serde(rename = "error")]
    Error { content: String },
    
    #[serde(rename = "done")]
    Done,
}

/// MCP SSE state
pub struct McpState {
    pub architect: RwLock<AiArchitect>,
}

/// Handler for SSE chat streaming
pub async fn mcp_stream_handler(
    State(state): State<Arc<McpState>>,
    Json(request): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!(message = %request.message, "MCP chat stream request");

    let message = request.message.clone();
    let state_clone = state.clone();

    let stream = async_stream::stream! {
        // Send thinking indicator
        yield Ok(Event::default().data(
            serde_json::to_string(&SseMessage::Thinking {
                content: "Processing your request...".to_string(),
            }).unwrap()
        ));

        // Get response from architect
        let mut arch = state_clone.architect.write().await;
        
        match arch.chat(&message).await {
            Ok(response) => {
                // Send content
                yield Ok(Event::default().data(
                    serde_json::to_string(&SseMessage::Content {
                        content: response,
                    }).unwrap()
                ));
            }
            Err(e) => {
                error!(error = %e, "AI Architect error");
                yield Ok(Event::default().data(
                    serde_json::to_string(&SseMessage::Error {
                        content: format!("Error: {}", e),
                    }).unwrap()
                ));
            }
        }

        // Send done signal
        yield Ok(Event::default().data(
            serde_json::to_string(&SseMessage::Done).unwrap()
        ));
    };

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping")
    )
}

/// Handler for listing available tools
pub async fn list_tools_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "tools": [
            {
                "name": "deploy_route",
                "description": "Deploy a new integration route to the gateway",
                "parameters": {
                    "path": { "type": "string", "description": "Route path" },
                    "method": { "type": "string", "description": "HTTP method" },
                    "upstream": { "type": "string", "description": "Upstream URL" },
                    "transform_script": { "type": "string", "description": "Rhai script", "optional": true }
                }
            },
            {
                "name": "lookup_schema",
                "description": "Search for API endpoints in knowledge base",
                "parameters": {
                    "query": { "type": "string", "description": "Search query" },
                    "limit": { "type": "integer", "description": "Max results", "optional": true }
                }
            },
            {
                "name": "validate_script",
                "description": "Validate a Rhai transformation script",
                "parameters": {
                    "script": { "type": "string", "description": "Script to validate" }
                }
            }
        ]
    }))
}

/// Handler for listing available prompts
pub async fn list_prompts_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "prompts": [
            {
                "name": "create_integration",
                "description": "Create a new integration between two services",
                "template": "Create a route that connects {{source}} to {{target}}"
            },
            {
                "name": "transform_data",
                "description": "Add data transformation to a route",
                "template": "Add a transformation to {{route}} that {{transformation}}"
            }
        ]
    }))
}

/// Handler for clearing conversation history
pub async fn clear_history_handler(
    State(state): State<Arc<McpState>>,
) -> Json<serde_json::Value> {
    let mut architect = state.architect.write().await;
    architect.clear_history();
    
    Json(serde_json::json!({
        "success": true,
        "message": "Conversation history cleared"
    }))
}

/// Handler for health check
pub async fn mcp_health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "mcp-architect",
        "version": "1.0.0"
    }))
}

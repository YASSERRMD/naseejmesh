//! AI Design handlers - Smart Design flow generation

use axum::{
    extract::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GenerateFlowRequest {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateFlowResponse {
    pub nodes: Vec<GeneratedNode>,
    pub edges: Vec<GeneratedEdge>,
}

#[derive(Debug, Serialize)]
pub struct GeneratedNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct GeneratedEdge {
    pub from_index: usize,
    pub to_index: usize,
}

// ============================================================================
// Handlers
// ============================================================================

/// Smart Design - Generate flow from natural language
/// POST /api/design/generate
pub async fn generate_flow(
    Json(request): Json<GenerateFlowRequest>,
) -> Result<Json<GenerateFlowResponse>, (StatusCode, String)> {
    info!(prompt = %request.prompt, "Generating flow with AI");

    // Parse request and generate flow based on keywords
    // In production, this would call Cohere/OpenAI API
    let nodes = parse_prompt_to_nodes(&request.prompt);
    let edges = generate_edges(&nodes);

    Ok(Json(GenerateFlowResponse { nodes, edges }))
}

/// Parse prompt and extract node types
fn parse_prompt_to_nodes(prompt: &str) -> Vec<GeneratedNode> {
    let prompt_lower = prompt.to_lowercase();
    let mut nodes = Vec::new();

    // Detect data sources
    if prompt_lower.contains("mqtt") || prompt_lower.contains("sensor") {
        nodes.push(GeneratedNode {
            node_type: "mqtt".to_string(),
            label: "MQTT Source".to_string(),
            config: serde_json::json!({
                "description": "MQTT data ingestion",
                "topic": "sensors/#"
            }),
        });
    }

    if prompt_lower.contains("http") || prompt_lower.contains("api") || prompt_lower.contains("rest") {
        nodes.push(GeneratedNode {
            node_type: "http".to_string(),
            label: "REST API".to_string(),
            config: serde_json::json!({
                "description": "HTTP API endpoint"
            }),
        });
    }

    // Detect processing
    if prompt_lower.contains("filter") || prompt_lower.contains("temperature") || prompt_lower.contains(">") {
        nodes.push(GeneratedNode {
            node_type: "filter".to_string(),
            label: "Data Filter".to_string(),
            config: serde_json::json!({
                "description": "Filter data by condition"
            }),
        });
    }

    if prompt_lower.contains("transform") || prompt_lower.contains("convert") {
        nodes.push(GeneratedNode {
            node_type: "transform".to_string(),
            label: "Transformer".to_string(),
            config: serde_json::json!({
                "description": "Transform data format"
            }),
        });
    }

    if prompt_lower.contains("ai") || prompt_lower.contains("llm") || prompt_lower.contains("analyze") {
        nodes.push(GeneratedNode {
            node_type: "ai".to_string(),
            label: "AI Processor".to_string(),
            config: serde_json::json!({
                "description": "AI/LLM processing",
                "model": "command-r-plus"
            }),
        });
    }

    if prompt_lower.contains("mcp") || prompt_lower.contains("tool") {
        nodes.push(GeneratedNode {
            node_type: "mcp".to_string(),
            label: "MCP Server".to_string(),
            config: serde_json::json!({
                "description": "Model Context Protocol"
            }),
        });
    }

    if prompt_lower.contains("split") || prompt_lower.contains("parallel") {
        nodes.push(GeneratedNode {
            node_type: "splitter".to_string(),
            label: "Splitter".to_string(),
            config: serde_json::json!({
                "description": "Split to parallel paths"
            }),
        });
    }

    if prompt_lower.contains("aggregate") || prompt_lower.contains("combine") || prompt_lower.contains("merge") {
        nodes.push(GeneratedNode {
            node_type: "aggregator".to_string(),
            label: "Aggregator".to_string(),
            config: serde_json::json!({
                "description": "Combine multiple inputs"
            }),
        });
    }

    // Detect destinations
    if prompt_lower.contains("postgres") || prompt_lower.contains("database") || prompt_lower.contains("save") || prompt_lower.contains("store") {
        nodes.push(GeneratedNode {
            node_type: "database".to_string(),
            label: "PostgreSQL".to_string(),
            config: serde_json::json!({
                "description": "Database storage",
                "address": "postgres://localhost:5432/data"
            }),
        });
    }

    // Fallback if no nodes detected
    if nodes.is_empty() {
        nodes.push(GeneratedNode {
            node_type: "http".to_string(),
            label: "API Gateway".to_string(),
            config: serde_json::json!({
                "description": "Entry point"
            }),
        });
        nodes.push(GeneratedNode {
            node_type: "transform".to_string(),
            label: "Processor".to_string(),
            config: serde_json::json!({
                "description": "Data processing"
            }),
        });
    }

    nodes
}

/// Generate edges connecting nodes sequentially
fn generate_edges(nodes: &[GeneratedNode]) -> Vec<GeneratedEdge> {
    let mut edges = Vec::new();
    for i in 0..nodes.len().saturating_sub(1) {
        edges.push(GeneratedEdge {
            from_index: i,
            to_index: i + 1,
        });
    }
    edges
}

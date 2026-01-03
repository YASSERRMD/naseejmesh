//! Cohere LLM Chat Service
//!
//! Provides chat completions using Cohere's Command models for the AI Architect.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};

/// LLM errors
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("No API key configured. Set COHERE_API_KEY environment variable.")]
    NoApiKey,

    #[error("Rate limited. Please retry after some time.")]
    RateLimited,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Tool definition for Cohere
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameter_definitions: serde_json::Value,
}

/// Tool call from Cohere
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub parameters: serde_json::Value,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Cohere chat request
#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    preamble: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    chat_history: Vec<ChatHistoryItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<&'a ToolDefinition>,
}

#[derive(Debug, Serialize)]
struct ChatHistoryItem {
    role: String,
    message: String,
}

/// Cohere chat response
#[derive(Debug, Deserialize)]
struct ChatResponse {
    text: String,
    #[serde(default)]
    tool_calls: Vec<ResponseToolCall>,
}

#[derive(Debug, Deserialize)]
struct ResponseToolCall {
    name: String,
    parameters: serde_json::Value,
}

/// Cohere LLM service
#[derive(Clone)]
pub struct CohereLlm {
    client: Client,
    api_key: Option<String>,
    model: String,
}

impl CohereLlm {
    /// Create a new Cohere LLM service
    pub fn new() -> Self {
        let api_key = env::var("COHERE_API_KEY").ok();

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            model: "command-r-plus".to_string(),
        }
    }

    /// Use a specific model
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// Check if API key is configured
    pub fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    /// Simple chat completion
    pub async fn chat(&self, message: &str) -> Result<String, LlmError> {
        self.chat_with_context(message, None, &[]).await
    }

    /// Chat with system prompt and history
    pub async fn chat_with_context(
        &self,
        message: &str,
        system_prompt: Option<&str>,
        history: &[ChatMessage],
    ) -> Result<String, LlmError> {
        let api_key = self.api_key.as_ref().ok_or(LlmError::NoApiKey)?;

        let chat_history: Vec<ChatHistoryItem> = history
            .iter()
            .map(|m| ChatHistoryItem {
                role: if m.role == "user" { "USER".to_string() } else { "CHATBOT".to_string() },
                message: m.content.clone(),
            })
            .collect();

        let request = ChatRequest {
            model: &self.model,
            message,
            preamble: system_prompt,
            chat_history,
            tools: Vec::new(),
        };

        debug!(model = %self.model, message_len = message.len(), "Calling Cohere chat API");

        let response = self
            .client
            .post("https://api.cohere.ai/v1/chat")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            warn!("Cohere API rate limited");
            return Err(LlmError::RateLimited);
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!("HTTP {}: {}", status, error_text)));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        info!(response_len = chat_response.text.len(), "Cohere chat response received");
        Ok(chat_response.text)
    }

    /// Chat with tool calling support
    pub async fn chat_with_tools(
        &self,
        message: &str,
        system_prompt: Option<&str>,
        history: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<(String, Vec<ToolCall>), LlmError> {
        let api_key = self.api_key.as_ref().ok_or(LlmError::NoApiKey)?;

        let chat_history: Vec<ChatHistoryItem> = history
            .iter()
            .map(|m| ChatHistoryItem {
                role: if m.role == "user" { "USER".to_string() } else { "CHATBOT".to_string() },
                message: m.content.clone(),
            })
            .collect();

        let tool_refs: Vec<&ToolDefinition> = tools.iter().collect();

        let request = ChatRequest {
            model: &self.model,
            message,
            preamble: system_prompt,
            chat_history,
            tools: tool_refs,
        };

        debug!(model = %self.model, tools = tools.len(), "Calling Cohere chat with tools");

        let response = self
            .client
            .post("https://api.cohere.ai/v1/chat")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(LlmError::RateLimited);
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!("HTTP {}: {}", status, error_text)));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        let tool_calls: Vec<ToolCall> = chat_response
            .tool_calls
            .into_iter()
            .map(|tc| ToolCall {
                name: tc.name,
                parameters: tc.parameters,
            })
            .collect();

        info!(
            response_len = chat_response.text.len(),
            tool_calls = tool_calls.len(),
            "Cohere chat response with tools"
        );

        Ok((chat_response.text, tool_calls))
    }
}

impl Default for CohereLlm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition_serialization() {
        let tool = ToolDefinition {
            name: "deploy_route".to_string(),
            description: "Deploy a new route".to_string(),
            parameter_definitions: serde_json::json!({
                "path": {"type": "string", "description": "Route path"},
                "upstream": {"type": "string", "description": "Upstream URL"}
            }),
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("deploy_route"));
    }

    #[test]
    fn test_is_available_without_key() {
        let llm = CohereLlm::new();
        // Just verify it doesn't panic
        let _ = llm.is_available();
    }

    #[test]
    fn test_model_selection() {
        let llm = CohereLlm::new().with_model("command-r");
        assert_eq!(llm.model, "command-r");
    }
}

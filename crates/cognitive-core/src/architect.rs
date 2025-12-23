//! AI Architect
//!
//! The NaseejArchitect is an AI agent that can design and deploy integrations
//! based on natural language descriptions. It uses tools to interact with
//! the gateway's configuration.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};

use crate::rhai_engine::RhaiEngine;
use crate::vector_store::VectorStore;
use crate::tools::{
    ToolRegistry, ToolResult, DeployRouteArgs, LookupSchemaArgs, ValidateRhaiArgs,
};

/// Architect errors
#[derive(Debug, Error)]
pub enum ArchitectError {
    #[error("LLM error: {0}")]
    LlmError(String),

    #[error("Tool error: {0}")]
    ToolError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Configuration for the AI Architect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectConfig {
    /// OpenAI API key
    pub api_key: Option<String>,

    /// Model to use (e.g., "gpt-4o", "gpt-3.5-turbo")
    #[serde(default = "default_model")]
    pub model: String,

    /// Temperature for generation (0.0 - 1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Maximum tokens in response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    2048
}

impl Default for ArchitectConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: default_model(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}

/// A conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// The AI Architect agent
pub struct NaseejArchitect {
    /// Configuration
    config: ArchitectConfig,

    /// Tool registry
    tools: Arc<ToolRegistry>,

    /// Conversation history
    history: Vec<Message>,

    /// System prompt
    system_prompt: String,
}

impl NaseejArchitect {
    /// Create a new architect
    pub fn new(
        config: ArchitectConfig,
        rhai_engine: Arc<RhaiEngine>,
        vector_store: Arc<RwLock<VectorStore>>,
    ) -> Self {
        let tools = Arc::new(ToolRegistry::new(rhai_engine, vector_store));

        let system_prompt = Self::build_system_prompt();

        Self {
            config,
            tools,
            history: Vec::new(),
            system_prompt,
        }
    }

    /// Build the system prompt for the AI
    fn build_system_prompt() -> String {
        let tool_descriptions = ToolRegistry::tool_descriptions();
        let tools_json = serde_json::to_string_pretty(&tool_descriptions).unwrap_or_default();

        format!(r#"You are the Naseej Architect, an AI assistant specialized in building high-performance integration routes.

Your role is to help users create API integrations, data transformations, and routing rules for the NaseejMesh gateway.

## Capabilities

You have access to the following tools:

{}

## Guidelines

1. **Understand the Request**: Carefully analyze what the user wants to achieve.
2. **Search for Existing APIs**: Use lookup_schema to find relevant API endpoints in the knowledge base.
3. **Design the Integration**: Create a route that connects the source to the destination.
4. **Write Transformations**: If data transformation is needed, write a Rhai script.
5. **Validate Before Deploy**: Always validate Rhai scripts before deploying.
6. **Deploy**: Use deploy_route to create the final integration.

## Rhai Scripting

Rhai is an embedded scripting language. Key features:
- Access `payload` (JSON object), `metadata` (map), `protocol`, `destination`
- Built-in functions: `parse_json()`, `to_json()`, `uuid()`, `timestamp()`, `now_utc()`
- Modify `payload` to transform data

Example Rhai script:
```rhai
// Convert temperature from Celsius to Fahrenheit
payload["temp_f"] = payload["temp"] * 9 / 5 + 32;
payload["converted_at"] = now_utc();
```

## Response Format

Always respond with:
1. **Analysis**: Brief understanding of the request
2. **Plan**: Steps you'll take
3. **Execution**: Tool calls and results
4. **Summary**: What was accomplished

Be concise and focus on actionable results."#, tools_json)
    }

    /// Process a user message and return a response
    pub async fn chat(&mut self, user_message: &str) -> Result<String, ArchitectError> {
        info!(message = %user_message, "Processing architect request");

        // Add user message to history
        self.history.push(Message {
            role: "user".to_string(),
            content: user_message.to_string(),
        });

        // Analyze the request and determine actions
        let actions = self.analyze_request(user_message).await?;

        // Execute tool calls
        let mut results = Vec::new();
        for action in actions {
            let result = self.execute_action(&action).await?;
            results.push((action, result));
        }

        // Generate response
        let response = self.generate_response(user_message, &results);

        // Add to history
        self.history.push(Message {
            role: "assistant".to_string(),
            content: response.clone(),
        });

        Ok(response)
    }

    /// Analyze a request and determine what tools to call
    async fn analyze_request(&self, message: &str) -> Result<Vec<ToolAction>, ArchitectError> {
        let mut actions = Vec::new();
        let message_lower = message.to_lowercase();

        // Simple keyword-based analysis (in production, this would use the LLM)
        
        // Check for schema lookup requests
        if message_lower.contains("find") || message_lower.contains("search") 
            || message_lower.contains("look") || message_lower.contains("what apis") {
            actions.push(ToolAction::LookupSchema {
                query: message.to_string(),
                limit: 5,
            });
        }

        // Check for route creation requests
        if message_lower.contains("create") || message_lower.contains("deploy")
            || message_lower.contains("route") || message_lower.contains("connect") {
            // Extract path and upstream from message (simplified)
            if let Some((path, upstream)) = self.extract_route_params(message) {
                actions.push(ToolAction::DeployRoute {
                    path,
                    upstream,
                    transform: None,
                });
            }
        }

        // Check for script validation requests
        if message_lower.contains("validate") || message_lower.contains("script")
            || message_lower.contains("rhai") {
            if let Some(script) = self.extract_script(message) {
                actions.push(ToolAction::ValidateRhai {
                    script,
                    test_payload: None,
                });
            }
        }

        debug!(actions = actions.len(), "Analyzed request");
        Ok(actions)
    }

    /// Extract route parameters from natural language
    fn extract_route_params(&self, message: &str) -> Option<(String, String)> {
        // Simple extraction - look for paths and URLs
        let path_regex = regex::Regex::new(r"/[\w/\-{}]+").ok()?;
        let url_regex = regex::Regex::new(r"https?://[\w\.\-:]+[\w/\-]*").ok()?;

        let path = path_regex.find(message).map(|m| m.as_str().to_string())?;
        let upstream = url_regex.find(message).map(|m| m.as_str().to_string())?;

        Some((path, upstream))
    }

    /// Extract a Rhai script from the message
    fn extract_script(&self, message: &str) -> Option<String> {
        // Look for code blocks
        if let Some(start) = message.find("```") {
            let after_start = &message[start + 3..];
            // Skip optional language identifier
            let content_start = after_start.find('\n').map(|i| i + 1).unwrap_or(0);
            let after_lang = &after_start[content_start..];
            
            if let Some(end) = after_lang.find("```") {
                return Some(after_lang[..end].trim().to_string());
            }
        }
        None
    }

    /// Execute a tool action
    async fn execute_action(&self, action: &ToolAction) -> Result<ToolResult, ArchitectError> {
        match action {
            ToolAction::LookupSchema { query, limit } => {
                self.tools.lookup_schema.call(LookupSchemaArgs {
                    query: query.clone(),
                    limit: *limit,
                }).await.pipe(Ok)
            }
            ToolAction::DeployRoute { path, upstream, transform } => {
                self.tools.deploy_route.call(DeployRouteArgs {
                    path: path.clone(),
                    upstream: upstream.clone(),
                    methods: vec!["GET".to_string(), "POST".to_string()],
                    transform_script: transform.clone(),
                    description: None,
                }).await.pipe(Ok)
            }
            ToolAction::ValidateRhai { script, test_payload } => {
                self.tools.validate_rhai.call(ValidateRhaiArgs {
                    script: script.clone(),
                    test_payload: test_payload.clone(),
                }).await.pipe(Ok)
            }
        }
    }

    /// Generate a response based on tool results
    fn generate_response(&self, _request: &str, results: &[(ToolAction, ToolResult)]) -> String {
        let mut response = String::new();

        if results.is_empty() {
            response.push_str("I understand your request, but I couldn't determine specific actions to take. ");
            response.push_str("Could you provide more details about what you'd like me to do?\n\n");
            response.push_str("I can help you:\n");
            response.push_str("- **Search for APIs**: \"Find APIs related to users\"\n");
            response.push_str("- **Create routes**: \"Create a route from /api/users to http://backend:8080\"\n");
            response.push_str("- **Validate scripts**: \"Validate this Rhai script: ```let x = 1;```\"\n");
            return response;
        }

        response.push_str("## Results\n\n");

        for (action, result) in results {
            let action_name = match action {
                ToolAction::LookupSchema { .. } => "Schema Lookup",
                ToolAction::DeployRoute { .. } => "Route Deployment",
                ToolAction::ValidateRhai { .. } => "Script Validation",
            };

            response.push_str(&format!("### {}\n\n", action_name));

            if result.success {
                response.push_str(&format!("✅ {}\n\n", result.message));
                if let Some(data) = &result.data {
                    response.push_str("```json\n");
                    response.push_str(&serde_json::to_string_pretty(data).unwrap_or_default());
                    response.push_str("\n```\n\n");
                }
            } else {
                response.push_str(&format!("❌ {}\n\n", result.message));
            }
        }

        response
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Get conversation history
    pub fn history(&self) -> &[Message] {
        &self.history
    }
}

/// A tool action to execute
#[derive(Debug, Clone)]
enum ToolAction {
    LookupSchema { query: String, limit: usize },
    DeployRoute { path: String, upstream: String, transform: Option<String> },
    ValidateRhai { script: String, test_payload: Option<serde_json::Value> },
}

/// Helper trait for chaining
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R where F: FnOnce(Self) -> R {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_architect_creation() {
        let rhai_engine = Arc::new(RhaiEngine::new());
        let vector_store = Arc::new(RwLock::new(VectorStore::new()));
        let config = ArchitectConfig::default();

        let architect = NaseejArchitect::new(config, rhai_engine, vector_store);
        assert!(architect.history().is_empty());
    }

    #[tokio::test]
    async fn test_extract_route_params() {
        let rhai_engine = Arc::new(RhaiEngine::new());
        let vector_store = Arc::new(RwLock::new(VectorStore::new()));
        let architect = NaseejArchitect::new(
            ArchitectConfig::default(),
            rhai_engine,
            vector_store,
        );

        let message = "Create a route from /api/users to http://localhost:8080/users";
        let params = architect.extract_route_params(message);
        
        assert!(params.is_some());
        let (path, upstream) = params.unwrap();
        assert_eq!(path, "/api/users");
        assert!(upstream.contains("localhost"));
    }

    #[tokio::test]
    async fn test_extract_script() {
        let rhai_engine = Arc::new(RhaiEngine::new());
        let vector_store = Arc::new(RwLock::new(VectorStore::new()));
        let architect = NaseejArchitect::new(
            ArchitectConfig::default(),
            rhai_engine,
            vector_store,
        );

        let message = "Validate this script:\n```rhai\nlet x = 1 + 2;\n```";
        let script = architect.extract_script(message);
        
        assert!(script.is_some());
        assert!(script.unwrap().contains("let x"));
    }

    #[tokio::test]
    async fn test_chat_basic() {
        let rhai_engine = Arc::new(RhaiEngine::new());
        let vector_store = Arc::new(RwLock::new(VectorStore::new()));
        let mut architect = NaseejArchitect::new(
            ArchitectConfig::default(),
            rhai_engine,
            vector_store,
        );

        let response = architect.chat("Hello, what can you do?").await.unwrap();
        assert!(!response.is_empty());
        assert_eq!(architect.history().len(), 2); // user + assistant
    }
}

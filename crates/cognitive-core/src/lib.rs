//! # Cognitive Core
//!
//! AI-driven cognitive control plane for NaseejMesh.
//!
//! This crate provides:
//! - `SchemaIngestor`: Parse and vectorize OpenAPI/WSDL specs
//! - `RhaiEngine`: Safe embedded scripting for transformations
//! - `NaseejArchitect`: AI agent with route deployment tools
//! - `McpServer`: MCP protocol interface for external AI tools
//! - `CohereEmbedding`: Text embeddings via Cohere API
//! - `EmbeddingProvider`: Unified embedding interface with caching
//! - `CohereLlm`: Chat completions via Cohere Command models

pub mod schema_ingestor;
pub mod rhai_engine;
pub mod architect;
pub mod mcp_server;
pub mod vector_store;
pub mod tools;
pub mod cohere_embedding;
pub mod embedding_provider;
pub mod cohere_llm;

pub use schema_ingestor::{SchemaIngestor, ApiEndpoint};
pub use rhai_engine::RhaiEngine;
pub use architect::{NaseejArchitect, ArchitectConfig};
pub use mcp_server::McpServer;
pub use vector_store::VectorStore;
pub use cohere_embedding::CohereEmbedding;
pub use embedding_provider::EmbeddingProvider;
pub use cohere_llm::{CohereLlm, ToolDefinition, ToolCall, ChatMessage};

//! # Cognitive Core
//!
//! AI-driven cognitive control plane for NaseejMesh.
//!
//! This crate provides:
//! - `SchemaIngestor`: Parse and vectorize OpenAPI/WSDL specs
//! - `RhaiEngine`: Safe embedded scripting for transformations
//! - `NaseejArchitect`: AI agent with route deployment tools
//! - `McpServer`: MCP protocol interface for external AI tools

pub mod schema_ingestor;
pub mod rhai_engine;
pub mod architect;
pub mod mcp_server;
pub mod vector_store;
pub mod tools;

pub use schema_ingestor::{SchemaIngestor, ApiEndpoint};
pub use rhai_engine::RhaiEngine;
pub use architect::NaseejArchitect;
pub use mcp_server::McpServer;
pub use vector_store::VectorStore;

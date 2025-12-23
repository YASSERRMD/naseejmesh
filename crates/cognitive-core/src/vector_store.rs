//! Vector Store for RAG
//!
//! Stores and retrieves API endpoint vectors from SurrealDB.
//! Enables semantic search for the AI Architect.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info};

use crate::schema_ingestor::ApiEndpoint;

/// Vector store errors
#[derive(Debug, Error)]
pub enum VectorError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

/// A stored vector entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    /// Unique identifier
    pub id: String,

    /// The text that was embedded
    pub text: String,

    /// Embedding vector (placeholder - actual vectors from OpenAI)
    pub vector: Vec<f32>,

    /// Associated metadata
    pub metadata: HashMap<String, String>,

    /// Source endpoint ID
    pub endpoint_id: String,
}

/// Vector store for API endpoints
pub struct VectorStore {
    /// Stored entries (in-memory for now, SurrealDB integration in production)
    entries: Vec<VectorEntry>,

    /// Embedding dimension
    embedding_dim: usize,
}

impl VectorStore {
    /// Create a new vector store
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            embedding_dim: 1536, // OpenAI text-embedding-3-small dimension
        }
    }

    /// Add an endpoint to the store
    pub async fn add_endpoint(&mut self, endpoint: &ApiEndpoint) -> Result<(), VectorError> {
        // Generate a simple placeholder embedding
        // In production, this would call OpenAI's embedding API
        let vector = self.generate_placeholder_embedding(&endpoint.embedding_text);

        let mut metadata = HashMap::new();
        metadata.insert("method".to_string(), endpoint.method.clone());
        metadata.insert("path".to_string(), endpoint.path.clone());
        if let Some(summary) = &endpoint.summary {
            metadata.insert("summary".to_string(), summary.clone());
        }

        let entry = VectorEntry {
            id: format!("vec-{}", endpoint.id),
            text: endpoint.embedding_text.clone(),
            vector,
            metadata,
            endpoint_id: endpoint.id.clone(),
        };

        self.entries.push(entry);

        debug!(endpoint_id = %endpoint.id, "Added endpoint to vector store");
        Ok(())
    }

    /// Search for similar endpoints
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, VectorError> {
        let query_vector = self.generate_placeholder_embedding(query);

        // Calculate cosine similarity with all entries
        let mut scored: Vec<(f32, &VectorEntry)> = self
            .entries
            .iter()
            .map(|entry| {
                let similarity = cosine_similarity(&query_vector, &entry.vector);
                (similarity, entry)
            })
            .collect();

        // Sort by similarity (descending)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        let results: Vec<SearchResult> = scored
            .into_iter()
            .take(limit)
            .map(|(score, entry)| SearchResult {
                endpoint_id: entry.endpoint_id.clone(),
                text: entry.text.clone(),
                score,
                metadata: entry.metadata.clone(),
            })
            .collect();

        info!(query = %query, results = results.len(), "Vector search completed");
        Ok(results)
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Generate a placeholder embedding (simple bag-of-words style)
    /// In production, this would use OpenAI's embedding API
    fn generate_placeholder_embedding(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; self.embedding_dim];
        
        // Simple hash-based embedding (placeholder)
        for (i, word) in text.split_whitespace().enumerate() {
            let hash = simple_hash(word);
            let idx = (hash as usize) % self.embedding_dim;
            vector[idx] += 1.0 / ((i + 1) as f32);
        }

        // Normalize
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in &mut vector {
                *v /= magnitude;
            }
        }

        vector
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Search result from vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Endpoint ID
    pub endpoint_id: String,

    /// Original text
    pub text: String,

    /// Similarity score (0.0 - 1.0)
    pub score: f32,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        dot / (mag_a * mag_b)
    }
}

/// Simple hash function for placeholder embeddings
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for c in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(c as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_store_add() {
        let mut store = VectorStore::new();
        
        let endpoint = ApiEndpoint {
            id: "test-1".to_string(),
            method: "GET".to_string(),
            path: "/users".to_string(),
            summary: Some("List users".to_string()),
            description: None,
            tags: vec!["users".to_string()],
            request_content_type: None,
            response_content_type: None,
            parameters: vec![],
            embedding_text: "GET /users List users".to_string(),
            source_spec: "test".to_string(),
        };

        store.add_endpoint(&endpoint).await.unwrap();
        assert_eq!(store.len(), 1);
    }

    #[tokio::test]
    async fn test_vector_search() {
        let mut store = VectorStore::new();

        // Add some endpoints
        let endpoints = vec![
            ("GET /users", "List all users"),
            ("POST /users", "Create a user"),
            ("GET /orders", "List orders"),
        ];

        for (i, (path, summary)) in endpoints.iter().enumerate() {
            let endpoint = ApiEndpoint {
                id: format!("test-{}", i),
                method: path.split_whitespace().next().unwrap().to_string(),
                path: path.split_whitespace().nth(1).unwrap().to_string(),
                summary: Some(summary.to_string()),
                description: None,
                tags: vec![],
                request_content_type: None,
                response_content_type: None,
                parameters: vec![],
                embedding_text: format!("{} {}", path, summary),
                source_spec: "test".to_string(),
            };
            store.add_endpoint(&endpoint).await.unwrap();
        }

        // Search for users
        let results = store.search("users", 2).await.unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 0.001);
    }
}

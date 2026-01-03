//! SurrealDB-backed Vector Store
//!
//! Persistent vector storage using SurrealDB for RAG pipeline.

use std::sync::Arc;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info};

use crate::vector_schema::VectorRecord;

/// Vector store errors
#[derive(Debug, Error)]
pub enum VectorError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),
}

/// Search result from vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub endpoint_id: String,
    pub text: String,
    pub score: f32,
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
}

/// SurrealDB-backed vector store
pub struct SurrealVectorStore<C: surrealdb::Connection> {
    db: Arc<surrealdb::Surreal<C>>,
    embedding_dim: usize,
}

impl<C: surrealdb::Connection> SurrealVectorStore<C> {
    /// Create a new SurrealDB vector store
    pub fn new(db: Arc<surrealdb::Surreal<C>>) -> Self {
        Self {
            db,
            embedding_dim: 1024, // Cohere embed-english-v3.0 dimension
        }
    }

    /// Add a vector record to the store
    pub async fn add(&self, record: VectorRecord) -> Result<(), VectorError> {
        let endpoint_id = record.endpoint_id.clone();
        let result: Result<Option<VectorRecord>, _> = self
            .db
            .create(("api_vectors", &endpoint_id))
            .content(record)
            .await;

        match result {
            Ok(_) => {
                debug!(endpoint_id = %endpoint_id, "Added vector to store");
                Ok(())
            }
            Err(e) => {
                if e.to_string().contains("already exists") {
                    Err(VectorError::DuplicateEntry(endpoint_id))
                } else {
                    Err(VectorError::DatabaseError(e.to_string()))
                }
            }
        }
    }

    /// Search for similar vectors using cosine similarity
    pub async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>, VectorError> {
        // Fetch all vectors (for small datasets)
        // For large datasets, use SurrealDB's vector search when available
        let records: Vec<VectorRecord> = self
            .db
            .select("api_vectors")
            .await
            .map_err(|e| VectorError::DatabaseError(e.to_string()))?;

        if records.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate cosine similarity with all records
        let mut scored: Vec<(f32, &VectorRecord)> = records
            .iter()
            .map(|r| {
                let sim = cosine_similarity(query_embedding, &r.embedding);
                (sim, r)
            })
            .collect();

        // Sort by similarity (descending)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        let results: Vec<SearchResult> = scored
            .into_iter()
            .take(limit)
            .map(|(score, r)| SearchResult {
                endpoint_id: r.endpoint_id.clone(),
                text: r.text.clone(),
                score,
                method: r.method.clone(),
                path: r.path.clone(),
                summary: r.summary.clone(),
            })
            .collect();

        info!(query_len = query_embedding.len(), results = results.len(), "Vector search completed");
        Ok(results)
    }

    /// Delete vectors by source specification
    pub async fn delete_by_source(&self, source_spec: &str) -> Result<usize, VectorError> {
        let source = source_spec.to_string();
        let mut result = self
            .db
            .query("DELETE api_vectors WHERE source_spec = $source RETURN BEFORE")
            .bind(("source", source.clone()))
            .await
            .map_err(|e| VectorError::DatabaseError(e.to_string()))?;

        let deleted: Vec<VectorRecord> = result
            .take(0)
            .map_err(|e| VectorError::DatabaseError(e.to_string()))?;

        info!(source = %source, count = deleted.len(), "Deleted vectors by source");
        Ok(deleted.len())
    }

    /// Get count of stored vectors
    pub async fn count(&self) -> Result<usize, VectorError> {
        let records: Vec<VectorRecord> = self
            .db
            .select("api_vectors")
            .await
            .map_err(|e| VectorError::DatabaseError(e.to_string()))?;
        Ok(records.len())
    }

    /// Clear all vectors
    pub async fn clear(&self) -> Result<(), VectorError> {
        self.db
            .query("DELETE api_vectors")
            .await
            .map_err(|e| VectorError::DatabaseError(e.to_string()))?;
        info!("Cleared all vectors from store");
        Ok(())
    }

    /// Get embedding dimension
    pub fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 0.001);
    }
}

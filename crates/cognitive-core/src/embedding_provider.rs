//! Unified Embedding Provider
//!
//! Provides a single interface for embeddings with caching and fallback support.

use std::time::Duration;
use moka::future::Cache;
use sha2::{Sha256, Digest};
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::cohere_embedding::{CohereEmbedding, EmbeddingError as CohereError};

/// Errors from the embedding provider
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

impl From<CohereError> for ProviderError {
    fn from(e: CohereError) -> Self {
        ProviderError::EmbeddingError(e.to_string())
    }
}

/// Unified embedding provider with caching
pub struct EmbeddingProvider {
    cohere: CohereEmbedding,
    cache: Cache<String, Vec<f32>>,
    use_placeholder: bool,
}

impl EmbeddingProvider {
    /// Create a new embedding provider
    pub fn new() -> Self {
        let cohere = CohereEmbedding::new();
        let use_placeholder = !cohere.is_available();

        if use_placeholder {
            warn!("COHERE_API_KEY not set, using placeholder embeddings");
        } else {
            info!("Cohere embedding service initialized");
        }

        Self {
            cohere,
            cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(3600 * 24)) // 24 hours
                .build(),
            use_placeholder,
        }
    }

    /// Check if real embeddings are available
    pub fn is_available(&self) -> bool {
        self.cohere.is_available()
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.cohere.dimension()
    }

    /// Embed a single text with caching
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, ProviderError> {
        let cache_key = self.hash_text(text);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            debug!("Cache hit for embedding");
            return Ok(cached);
        }

        // Generate embedding
        let embedding = if self.use_placeholder {
            self.cohere.placeholder_embedding(text)
        } else {
            match self.cohere.embed(text).await {
                Ok(e) => e,
                Err(CohereError::RateLimited) => {
                    warn!("Rate limited, retrying after delay");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    self.cohere.embed(text).await?
                }
                Err(CohereError::NoApiKey) => {
                    warn!("No API key, falling back to placeholder");
                    self.cohere.placeholder_embedding(text)
                }
                Err(e) => return Err(e.into()),
            }
        };

        // Cache the result
        self.cache.insert(cache_key, embedding.clone()).await;
        
        Ok(embedding)
    }

    /// Embed multiple texts with caching
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError> {
        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_texts = Vec::new();
        let mut uncached_indices = Vec::new();

        // Check cache for each text
        for (i, text) in texts.iter().enumerate() {
            let cache_key = self.hash_text(text);
            if let Some(cached) = self.cache.get(&cache_key).await {
                results.push((i, cached));
            } else {
                uncached_texts.push(text.clone());
                uncached_indices.push(i);
            }
        }

        // Embed uncached texts
        if !uncached_texts.is_empty() {
            let new_embeddings = if self.use_placeholder {
                uncached_texts
                    .iter()
                    .map(|t| self.cohere.placeholder_embedding(t))
                    .collect()
            } else {
                self.cohere.embed_batch(&uncached_texts).await?
            };

            // Cache and add to results
            for (idx, embedding) in uncached_indices.into_iter().zip(new_embeddings) {
                let cache_key = self.hash_text(&uncached_texts[results.len()]);
                self.cache.insert(cache_key, embedding.clone()).await;
                results.push((idx, embedding));
            }
        }

        // Sort by original index
        results.sort_by_key(|(i, _)| *i);
        
        Ok(results.into_iter().map(|(_, e)| e).collect())
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (u64, u64) {
        (self.cache.entry_count(), self.cache.weighted_size())
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        self.cache.invalidate_all();
        info!("Embedding cache cleared");
    }

    /// Hash text for cache key
    fn hash_text(&self, text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl Default for EmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_placeholder_embedding() {
        let provider = EmbeddingProvider::new();
        let embedding = provider.embed("Hello world").await.unwrap();
        assert_eq!(embedding.len(), provider.dimension());
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let provider = EmbeddingProvider::new();
        
        // First call
        let _ = provider.embed("Test text").await.unwrap();
        
        // Second call should be cached
        let _ = provider.embed("Test text").await.unwrap();
        
        let (count, _) = provider.cache_stats();
        assert!(count >= 1);
    }

    #[test]
    fn test_hash_deterministic() {
        let provider = EmbeddingProvider::new();
        let hash1 = provider.hash_text("test");
        let hash2 = provider.hash_text("test");
        assert_eq!(hash1, hash2);
    }
}

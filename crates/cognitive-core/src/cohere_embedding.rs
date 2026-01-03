//! Cohere Embedding Service
//!
//! Provides text embeddings using Cohere's embed-english-v3.0 model.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Embedding service errors
#[derive(Debug, Error)]
pub enum EmbeddingError {
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

/// Cohere embedding request
#[derive(Debug, Serialize)]
struct EmbedRequest<'a> {
    texts: &'a [String],
    model: &'a str,
    input_type: &'a str,
    truncate: &'a str,
}

/// Cohere embedding response
#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

/// Cohere error response
#[derive(Debug, Deserialize)]
struct CohereError {
    message: String,
}

/// Cohere embedding service
#[derive(Clone)]
pub struct CohereEmbedding {
    client: Client,
    api_key: Option<String>,
    model: String,
    dimension: usize,
}

impl CohereEmbedding {
    /// Create a new Cohere embedding service
    pub fn new() -> Self {
        let api_key = env::var("COHERE_API_KEY").ok();
        
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            model: "embed-english-v3.0".to_string(),
            dimension: 1024,
        }
    }

    /// Create with a specific model
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        // Update dimension based on model
        self.dimension = match model {
            "embed-english-v3.0" => 1024,
            "embed-multilingual-v3.0" => 1024,
            "embed-english-light-v3.0" => 384,
            "embed-multilingual-light-v3.0" => 384,
            _ => 1024,
        };
        self
    }

    /// Check if API key is configured
    pub fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get embedding dimension for current model
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Embed a single text
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InvalidResponse("Empty response".into()))
    }

    /// Embed multiple texts in a single API call (max 96 texts)
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Cohere has a limit of 96 texts per request
        if texts.len() > 96 {
            return self.embed_batch_chunked(texts).await;
        }

        self.embed_batch_internal(texts).await
    }

    /// Internal batch embed without size checking (max 96)
    async fn embed_batch_internal(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let api_key = self.api_key.as_ref().ok_or(EmbeddingError::NoApiKey)?;

        let request = EmbedRequest {
            texts,
            model: &self.model,
            input_type: "search_document",
            truncate: "END",
        };

        debug!(texts = texts.len(), model = %self.model, "Calling Cohere embed API");

        let response = self
            .client
            .post("https://api.cohere.ai/v1/embed")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| EmbeddingError::NetworkError(e.to_string()))?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            warn!("Cohere API rate limited");
            return Err(EmbeddingError::RateLimited);
        }

        if !status.is_success() {
            let error: CohereError = response
                .json()
                .await
                .unwrap_or(CohereError { message: format!("HTTP {}", status) });
            return Err(EmbeddingError::ApiError(error.message));
        }

        let embed_response: EmbedResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::InvalidResponse(e.to_string()))?;

        info!(embeddings = embed_response.embeddings.len(), "Cohere embeddings received");
        Ok(embed_response.embeddings)
    }

    /// Embed texts in chunks for large batches
    async fn embed_batch_chunked(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut all_embeddings = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(96) {
            let chunk_vec: Vec<String> = chunk.to_vec();
            let embeddings = self.embed_batch_internal(&chunk_vec).await?;
            all_embeddings.extend(embeddings);

            // Small delay to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(all_embeddings)
    }

    /// Generate a placeholder embedding for offline mode
    pub fn placeholder_embedding(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; self.dimension];

        // Simple hash-based embedding (for development without API key)
        for (i, word) in text.split_whitespace().enumerate() {
            let hash = simple_hash(word);
            let idx = (hash as usize) % self.dimension;
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

impl Default for CohereEmbedding {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn test_placeholder_embedding_dimension() {
        let service = CohereEmbedding::new();
        let embedding = service.placeholder_embedding("Hello world");
        assert_eq!(embedding.len(), 1024);
    }

    #[test]
    fn test_placeholder_embedding_normalized() {
        let service = CohereEmbedding::new();
        let embedding = service.placeholder_embedding("Test text for normalization");
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_model_dimensions() {
        let service = CohereEmbedding::new().with_model("embed-english-light-v3.0");
        assert_eq!(service.dimension(), 384);

        let service = CohereEmbedding::new().with_model("embed-english-v3.0");
        assert_eq!(service.dimension(), 1024);
    }

    #[test]
    fn test_is_available_without_key() {
        // In tests, COHERE_API_KEY is typically not set
        let service = CohereEmbedding::new();
        // This will be false unless env var is set
        // Just checking it doesn't panic
        let _ = service.is_available();
    }
}

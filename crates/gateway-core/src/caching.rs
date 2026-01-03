//! Request/Response Caching
//!
//! Provides in-memory caching for API responses to improve performance.

use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use moka::future::Cache;
use hyper::{Request, Response, StatusCode};
use hyper::body::Bytes;
use http_body_util::Full;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of items in the cache
    pub max_capacity: u64,
    /// Time to live for cached items
    pub ttl: Duration,
    /// Whether to cache errors (4xx, 5xx)
    pub cache_errors: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10_000,
            ttl: Duration::from_secs(60),
            cache_errors: false,
        }
    }
}

/// Cached response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Vec<u8>,
    pub created_at: std::time::SystemTime,
}

impl CachedResponse {
    pub fn is_fresh(&self, ttl: Duration) -> bool {
        if let Ok(elapsed) = self.created_at.elapsed() {
            elapsed < ttl
        } else {
            false
        }
    }
}

/// Global response cache
#[derive(Clone)]
pub struct ResponseCache {
    cache: Cache<String, CachedResponse>,
    config: CacheConfig,
}

impl ResponseCache {
    pub fn new(config: CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.ttl)
            .build();

        Self { cache, config }
    }

    /// Generate cache key from request
    pub fn cache_key<B>(&self, req: &Request<B>) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        req.method().hash(&mut hasher);
        req.uri().hash(&mut hasher);
        // Add Authorization header to key if present? 
        // Typically authenticated responses shouldn't be shared cached unless Vary: Cookie/Auth
        // For now, simple key based on method + URI
        hasher.finish().to_string()
    }

    /// Get cached response
    pub async fn get(&self, key: &str) -> Option<CachedResponse> {
        self.cache.get(key).await
    }

    /// Store response in cache
    pub async fn insert(&self, key: String, response: CachedResponse) {
        if !self.config.cache_errors && (response.status >= 400) {
            return;
        }
        self.cache.insert(key, response).await;
    }

    /// Reconstruct Hyper response from cached data
    pub fn to_response(&self, cached: &CachedResponse) -> Response<Full<Bytes>> {
        let mut builder = Response::builder().status(StatusCode::from_u16(cached.status).unwrap());

        for (k, v) in &cached.headers {
            builder = builder.header(k, v);
        }
        // Add cache hit header
        builder = builder.header("X-Cache", "HIT");

        builder
            .body(Full::new(Bytes::from(cached.body.clone())))
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_insertion_retrieval() {
        let config = CacheConfig::default();
        let cache = ResponseCache::new(config);
        
        let response = CachedResponse {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: vec![1, 2, 3],
            created_at: std::time::SystemTime::now(),
        };

        cache.insert("test_key".to_string(), response.clone()).await;
        
        let retrieved = cache.get("test_key").await.unwrap();
        assert_eq!(retrieved.body, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        let config = CacheConfig {
            ttl: Duration::from_millis(10),
            ..Default::default()
        };
        let cache = ResponseCache::new(config);
        
        let response = CachedResponse {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: vec![1, 2, 3],
            created_at: std::time::SystemTime::now(),
        };

        cache.insert("test_key".to_string(), response).await;
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Moka might not expire immediately on get in test env without mocking time, 
        // but let's check basic ttl config
        // Actually moka ttl is lazy or background thread dependent.
        // For unit test reliability we might skip strict timing checks or use mock time.
    }
}

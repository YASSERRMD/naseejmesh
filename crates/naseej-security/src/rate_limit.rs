//! Distributed Rate Limiting
//!
//! Token bucket rate limiter with support for distributed state sync.
//! Uses DashMap for lock-free concurrent access.

use dashmap::DashMap;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, warn};

/// Rate limiting errors
#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded")]
    Exceeded { retry_after_ms: u64 },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Rate limit check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,

    /// Remaining tokens
    pub remaining: u64,

    /// Total limit
    pub limit: u64,

    /// Time until limit resets (ms)
    pub reset_after_ms: u64,

    /// Retry after (ms) - only set if blocked
    pub retry_after_ms: Option<u64>,
}

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per window
    pub requests_per_window: u64,

    /// Window duration in seconds
    pub window_secs: u64,

    /// Burst allowance (extra tokens for bursty traffic)
    pub burst_size: u64,

    /// Enable distributed sync
    pub distributed: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 100,
            window_secs: 60,
            burst_size: 10,
            distributed: false,
        }
    }
}

/// Token bucket state
#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    config: RateLimitConfig,
}

impl TokenBucket {
    fn new(config: RateLimitConfig) -> Self {
        let tokens = (config.requests_per_window + config.burst_size) as f64;
        Self {
            tokens,
            last_update: Instant::now(),
            config,
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        
        // Calculate refill rate (tokens per second)
        let refill_rate = self.config.requests_per_window as f64 / self.config.window_secs as f64;
        
        // Add tokens
        let max_tokens = (self.config.requests_per_window + self.config.burst_size) as f64;
        self.tokens = (self.tokens + elapsed * refill_rate).min(max_tokens);
        self.last_update = now;
    }

    /// Try to consume a token
    fn try_consume(&mut self, tokens: f64) -> RateLimitResult {
        self.refill();

        let max_tokens = (self.config.requests_per_window + self.config.burst_size) as f64;

        if self.tokens >= tokens {
            self.tokens -= tokens;
            
            // Calculate reset time
            let tokens_needed = max_tokens - self.tokens;
            let refill_rate = self.config.requests_per_window as f64 / self.config.window_secs as f64;
            let reset_after_ms = ((tokens_needed / refill_rate) * 1000.0) as u64;

            RateLimitResult {
                allowed: true,
                remaining: self.tokens as u64,
                limit: self.config.requests_per_window,
                reset_after_ms,
                retry_after_ms: None,
            }
        } else {
            // Calculate retry time
            let tokens_needed = tokens - self.tokens;
            let refill_rate = self.config.requests_per_window as f64 / self.config.window_secs as f64;
            let retry_after_ms = ((tokens_needed / refill_rate) * 1000.0).ceil() as u64;

            RateLimitResult {
                allowed: false,
                remaining: 0,
                limit: self.config.requests_per_window,
                reset_after_ms: (self.config.window_secs * 1000),
                retry_after_ms: Some(retry_after_ms),
            }
        }
    }
}

/// Rate limiter with per-key buckets
pub struct RateLimiter {
    buckets: DashMap<String, Mutex<TokenBucket>>,
    default_config: RateLimitConfig,
    key_configs: DashMap<String, RateLimitConfig>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(default_config: RateLimitConfig) -> Self {
        Self {
            buckets: DashMap::new(),
            default_config,
            key_configs: DashMap::new(),
        }
    }

    /// Set config for a specific key
    pub fn set_key_config(&self, key: impl Into<String>, config: RateLimitConfig) {
        self.key_configs.insert(key.into(), config);
    }

    /// Check rate limit for a key
    pub fn check(&self, key: &str) -> RateLimitResult {
        self.check_with_cost(key, 1.0)
    }

    /// Check rate limit with custom cost
    pub fn check_with_cost(&self, key: &str, cost: f64) -> RateLimitResult {
        // Get or create bucket
        let bucket = self.buckets.entry(key.to_string()).or_insert_with(|| {
            let config = self
                .key_configs
                .get(key)
                .map(|c| c.clone())
                .unwrap_or_else(|| self.default_config.clone());
            Mutex::new(TokenBucket::new(config))
        });

        let result = bucket.lock().try_consume(cost);

        if !result.allowed {
            warn!(
                key = %key,
                retry_after_ms = ?result.retry_after_ms,
                "Rate limit exceeded"
            );
        } else {
            debug!(
                key = %key,
                remaining = result.remaining,
                "Rate limit check passed"
            );
        }

        result
    }

    /// Get current state for a key
    pub fn get_state(&self, key: &str) -> Option<RateLimitResult> {
        self.buckets.get(key).map(|bucket| {
            let mut guard = bucket.lock();
            guard.refill();
            
            let max_tokens = (guard.config.requests_per_window + guard.config.burst_size) as f64;
            let refill_rate = guard.config.requests_per_window as f64 / guard.config.window_secs as f64;
            let tokens_needed = max_tokens - guard.tokens;
            let reset_after_ms = ((tokens_needed / refill_rate) * 1000.0) as u64;

            RateLimitResult {
                allowed: true,
                remaining: guard.tokens as u64,
                limit: guard.config.requests_per_window,
                reset_after_ms,
                retry_after_ms: None,
            }
        })
    }

    /// Clear expired buckets (for memory management)
    pub fn cleanup_expired(&self, max_idle_secs: u64) {
        let now = Instant::now();
        let max_idle = Duration::from_secs(max_idle_secs);

        self.buckets.retain(|_, bucket| {
            let guard = bucket.lock();
            now.duration_since(guard.last_update) < max_idle
        });
    }

    /// Get statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.buckets.len(), self.key_configs.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rate_limit() {
        let config = RateLimitConfig {
            requests_per_window: 5,
            window_secs: 60,
            burst_size: 0,
            distributed: false,
        };

        let limiter = RateLimiter::new(config);

        // First 5 requests should succeed
        for i in 0..5 {
            let result = limiter.check("user1");
            assert!(result.allowed, "Request {} should be allowed", i);
        }

        // 6th request should fail
        let result = limiter.check("user1");
        assert!(!result.allowed);
        assert!(result.retry_after_ms.is_some());
    }

    #[test]
    fn test_burst_allowance() {
        let config = RateLimitConfig {
            requests_per_window: 5,
            window_secs: 60,
            burst_size: 3,
            distributed: false,
        };

        let limiter = RateLimiter::new(config);

        // Should allow 8 requests (5 + 3 burst)
        for i in 0..8 {
            let result = limiter.check("user2");
            assert!(result.allowed, "Request {} should be allowed", i);
        }

        // 9th should fail
        let result = limiter.check("user2");
        assert!(!result.allowed);
    }

    #[test]
    fn test_per_key_isolation() {
        let config = RateLimitConfig {
            requests_per_window: 3,
            window_secs: 60,
            burst_size: 0,
            distributed: false,
        };

        let limiter = RateLimiter::new(config);

        // Exhaust user1
        for _ in 0..3 {
            limiter.check("user1");
        }
        assert!(!limiter.check("user1").allowed);

        // user2 should still have quota
        assert!(limiter.check("user2").allowed);
    }

    #[test]
    fn test_custom_key_config() {
        let default_config = RateLimitConfig {
            requests_per_window: 10,
            window_secs: 60,
            burst_size: 0,
            distributed: false,
        };

        let limiter = RateLimiter::new(default_config);

        // Set higher limit for premium user
        limiter.set_key_config("premium_user", RateLimitConfig {
            requests_per_window: 100,
            window_secs: 60,
            burst_size: 20,
            distributed: false,
        });

        // Regular user
        for _ in 0..10 {
            limiter.check("regular_user");
        }
        assert!(!limiter.check("regular_user").allowed);

        // Premium user should still have quota
        for _ in 0..50 {
            assert!(limiter.check("premium_user").allowed);
        }
    }
}

//! Test scenarios for integration testing

use crate::fixtures::{PayloadGenerator, RouteFixture};
use crate::http_client::TestClient;
use naseej_security::{WafConfig, WafEngine, RateLimiter, RateLimitConfig};
use std::time::Instant;

/// WAF security test scenarios
pub struct WafScenarios;

impl WafScenarios {
    /// Test SQL injection blocking
    pub fn test_sql_injection_blocking(engine: &WafEngine) -> Vec<(String, bool)> {
        PayloadGenerator::sql_injection_payloads()
            .into_iter()
            .map(|payload| {
                let result = engine.scan(&payload);
                (payload, !result.allowed)
            })
            .collect()
    }

    /// Test XSS blocking
    pub fn test_xss_blocking(engine: &WafEngine) -> Vec<(String, bool)> {
        PayloadGenerator::xss_payloads()
            .into_iter()
            .map(|payload| {
                let result = engine.scan(&payload);
                (payload, !result.allowed)
            })
            .collect()
    }

    /// Test path traversal blocking
    pub fn test_path_traversal_blocking(engine: &WafEngine) -> Vec<(String, bool)> {
        PayloadGenerator::path_traversal_payloads()
            .into_iter()
            .map(|payload| {
                let result = engine.scan(&payload);
                (payload, !result.allowed)
            })
            .collect()
    }

    /// Test legitimate payloads are allowed
    pub fn test_legitimate_traffic(engine: &WafEngine) -> Vec<(String, bool)> {
        let payloads = vec![
            r#"{"name": "John Doe", "email": "john@example.com"}"#.to_string(),
            r#"{"count": 42, "active": true}"#.to_string(),
            "Hello, this is a normal message".to_string(),
            "The quick brown fox jumps over the lazy dog".to_string(),
        ];

        payloads
            .into_iter()
            .map(|payload| {
                let result = engine.scan(&payload);
                (payload, result.allowed)
            })
            .collect()
    }
}

/// Rate limiting test scenarios
pub struct RateLimitScenarios;

impl RateLimitScenarios {
    /// Test basic rate limiting
    pub fn test_basic_limiting(limit: u64) -> (u64, u64) {
        let config = RateLimitConfig {
            requests_per_window: limit,
            window_secs: 60,
            burst_size: 0,
            distributed: false,
        };

        let limiter = RateLimiter::new(config);
        let mut allowed = 0;
        let mut blocked = 0;

        for _ in 0..(limit + 10) {
            let result = limiter.check("test_client");
            if result.allowed {
                allowed += 1;
            } else {
                blocked += 1;
            }
        }

        (allowed, blocked)
    }

    /// Test burst handling
    pub fn test_burst_handling(limit: u64, burst: u64) -> u64 {
        let config = RateLimitConfig {
            requests_per_window: limit,
            window_secs: 60,
            burst_size: burst,
            distributed: false,
        };

        let limiter = RateLimiter::new(config);
        let mut allowed = 0;

        for _ in 0..(limit + burst + 10) {
            let result = limiter.check("burst_client");
            if result.allowed {
                allowed += 1;
            }
        }

        allowed
    }

    /// Test per-client isolation
    pub fn test_client_isolation(limit: u64) -> bool {
        let config = RateLimitConfig {
            requests_per_window: limit,
            window_secs: 60,
            burst_size: 0,
            distributed: false,
        };

        let limiter = RateLimiter::new(config);

        // Exhaust client A
        for _ in 0..limit {
            limiter.check("client_a");
        }

        // Client A should be blocked
        let a_blocked = !limiter.check("client_a").allowed;

        // Client B should still be allowed
        let b_allowed = limiter.check("client_b").allowed;

        a_blocked && b_allowed
    }
}

/// Performance benchmark scenarios
pub struct PerformanceScenarios;

impl PerformanceScenarios {
    /// Benchmark WAF scan performance
    pub fn benchmark_waf_scan(iterations: usize) -> (u64, u64, u64) {
        let engine = WafEngine::new(WafConfig::default()).unwrap();
        let payload = PayloadGenerator::random_json(3, 5).to_string();

        let mut times: Vec<u64> = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = engine.scan(&payload);
            times.push(start.elapsed().as_micros() as u64);
        }

        times.sort();
        let min = *times.first().unwrap_or(&0);
        let max = *times.last().unwrap_or(&0);
        let avg = times.iter().sum::<u64>() / iterations as u64;

        (min, avg, max)
    }

    /// Benchmark rate limiter performance
    pub fn benchmark_rate_limiter(iterations: usize) -> (u64, u64, u64) {
        let limiter = RateLimiter::new(RateLimitConfig {
            requests_per_window: 1_000_000, // High limit for benchmark
            window_secs: 60,
            burst_size: 0,
            distributed: false,
        });

        let mut times: Vec<u64> = Vec::with_capacity(iterations);

        for i in 0..iterations {
            let key = format!("client_{}", i % 100);
            let start = Instant::now();
            let _ = limiter.check(&key);
            times.push(start.elapsed().as_nanos() as u64);
        }

        times.sort();
        let min = *times.first().unwrap_or(&0);
        let max = *times.last().unwrap_or(&0);
        let avg = times.iter().sum::<u64>() / iterations as u64;

        (min, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waf_sql_injection_blocking() {
        let engine = WafEngine::new(WafConfig::default()).unwrap();
        let results = WafScenarios::test_sql_injection_blocking(&engine);

        for (payload, blocked) in &results {
            assert!(blocked, "SQL injection should be blocked: {}", payload);
        }
    }

    #[test]
    fn test_waf_legitimate_traffic() {
        let engine = WafEngine::new(WafConfig::default()).unwrap();
        let results = WafScenarios::test_legitimate_traffic(&engine);

        for (payload, allowed) in &results {
            assert!(allowed, "Legitimate traffic should be allowed: {}", payload);
        }
    }

    #[test]
    fn test_rate_limit_basic() {
        let (allowed, blocked) = RateLimitScenarios::test_basic_limiting(10);
        assert_eq!(allowed, 10);
        assert_eq!(blocked, 10);
    }

    #[test]
    fn test_rate_limit_burst() {
        let allowed = RateLimitScenarios::test_burst_handling(10, 5);
        assert_eq!(allowed, 15); // limit + burst
    }

    #[test]
    fn test_rate_limit_isolation() {
        let isolated = RateLimitScenarios::test_client_isolation(5);
        assert!(isolated);
    }

    #[test]
    fn test_waf_performance() {
        let (min, avg, max) = PerformanceScenarios::benchmark_waf_scan(1000);
        println!("WAF scan: min={}us, avg={}us, max={}us", min, avg, max);
        assert!(avg < 1000, "WAF scan should be < 1ms average");
    }

    #[test]
    fn test_rate_limiter_performance() {
        let (min, avg, max) = PerformanceScenarios::benchmark_rate_limiter(10000);
        println!("Rate limit: min={}ns, avg={}ns, max={}ns", min, avg, max);
        assert!(avg < 10000, "Rate limit check should be < 10us average");
    }
}

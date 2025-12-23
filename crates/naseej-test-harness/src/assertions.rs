//! Response assertions for testing

use reqwest::Response;
use serde_json::Value;

/// Response assertion helpers
pub struct ResponseAssertions;

impl ResponseAssertions {
    /// Assert response is successful (2xx)
    pub fn assert_success(status: u16) {
        assert!(
            (200..300).contains(&status),
            "Expected success status, got {}",
            status
        );
    }

    /// Assert response is client error (4xx)
    pub fn assert_client_error(status: u16) {
        assert!(
            (400..500).contains(&status),
            "Expected client error status, got {}",
            status
        );
    }

    /// Assert response is server error (5xx)
    pub fn assert_server_error(status: u16) {
        assert!(
            (500..600).contains(&status),
            "Expected server error status, got {}",
            status
        );
    }

    /// Assert JSON contains key
    pub fn assert_json_has_key(json: &Value, key: &str) {
        assert!(
            json.get(key).is_some(),
            "Expected JSON to have key '{}', got {:?}",
            key,
            json
        );
    }

    /// Assert JSON field equals value
    pub fn assert_json_field_eq(json: &Value, key: &str, expected: &Value) {
        let actual = json.get(key);
        assert_eq!(
            actual,
            Some(expected),
            "Expected JSON field '{}' to equal {:?}, got {:?}",
            key,
            expected,
            actual
        );
    }

    /// Assert JSON is array with minimum length
    pub fn assert_json_array_min_length(json: &Value, min_len: usize) {
        let arr = json.as_array().expect("Expected JSON array");
        assert!(
            arr.len() >= min_len,
            "Expected array with at least {} elements, got {}",
            min_len,
            arr.len()
        );
    }

    /// Assert response header exists
    pub async fn assert_header_exists(response: &Response, header: &str) {
        assert!(
            response.headers().contains_key(header),
            "Expected header '{}' to exist",
            header
        );
    }

    /// Assert response header value
    pub fn assert_header_value(headers: &reqwest::header::HeaderMap, header: &str, expected: &str) {
        let value = headers
            .get(header)
            .expect(&format!("Header '{}' not found", header))
            .to_str()
            .expect("Invalid header value");
        assert_eq!(
            value, expected,
            "Expected header '{}' to be '{}', got '{}'",
            header, expected, value
        );
    }

    /// Assert response time is within limit
    pub fn assert_response_time_ms(elapsed_ms: u64, max_ms: u64) {
        assert!(
            elapsed_ms <= max_ms,
            "Expected response time <= {}ms, got {}ms",
            max_ms,
            elapsed_ms
        );
    }
}

/// WAF-specific assertions
pub struct WafAssertions;

impl WafAssertions {
    /// Assert payload is blocked
    pub fn assert_blocked(status: u16) {
        assert_eq!(
            status, 403,
            "Expected WAF block (403), got {}",
            status
        );
    }

    /// Assert payload is allowed
    pub fn assert_allowed(status: u16) {
        assert_ne!(
            status, 403,
            "Expected request to be allowed, got 403 (blocked)"
        );
    }
}

/// Rate limit assertions
pub struct RateLimitAssertions;

impl RateLimitAssertions {
    /// Assert rate limited
    pub fn assert_rate_limited(status: u16) {
        assert_eq!(
            status, 429,
            "Expected rate limit (429), got {}",
            status
        );
    }

    /// Assert not rate limited
    pub fn assert_not_rate_limited(status: u16) {
        assert_ne!(
            status, 429,
            "Expected request to not be rate limited, got 429"
        );
    }

    /// Assert retry-after header
    pub fn assert_retry_after(headers: &reqwest::header::HeaderMap) {
        assert!(
            headers.contains_key("retry-after"),
            "Expected Retry-After header for rate limited response"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_success() {
        ResponseAssertions::assert_success(200);
        ResponseAssertions::assert_success(201);
        ResponseAssertions::assert_success(204);
    }

    #[test]
    #[should_panic]
    fn test_assert_success_fails_on_error() {
        ResponseAssertions::assert_success(404);
    }

    #[test]
    fn test_json_assertions() {
        let json = serde_json::json!({"name": "test", "count": 42});
        ResponseAssertions::assert_json_has_key(&json, "name");
        ResponseAssertions::assert_json_field_eq(&json, "count", &serde_json::json!(42));
    }

    #[test]
    fn test_waf_assertions() {
        WafAssertions::assert_blocked(403);
        WafAssertions::assert_allowed(200);
    }

    #[test]
    fn test_rate_limit_assertions() {
        RateLimitAssertions::assert_rate_limited(429);
        RateLimitAssertions::assert_not_rate_limited(200);
    }
}

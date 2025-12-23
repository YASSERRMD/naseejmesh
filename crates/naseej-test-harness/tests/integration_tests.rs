//! Integration tests for NaseejMesh
//!
//! Run with: cargo test --test integration_tests

use naseej_test_harness::fixtures::{PayloadGenerator, RouteFixture};
use naseej_test_harness::scenarios::{WafScenarios, RateLimitScenarios, PerformanceScenarios};
use naseej_security::{WafConfig, WafEngine, RateLimiter, RateLimitConfig};

/// Test WAF blocks all SQL injection attempts
#[test]
fn test_waf_blocks_sql_injection() {
    let engine = WafEngine::new(WafConfig::default()).unwrap();
    let results = WafScenarios::test_sql_injection_blocking(&engine);

    let blocked_count = results.iter().filter(|(_, blocked)| *blocked).count();
    let total_count = results.len();

    println!("SQL Injection: {}/{} blocked", blocked_count, total_count);

    // All SQL injections should be blocked
    for (payload, blocked) in &results {
        assert!(blocked, "SQL injection should be blocked: {}", payload);
    }
}

/// Test WAF blocks XSS attempts
#[test]
fn test_waf_blocks_xss() {
    let engine = WafEngine::new(WafConfig::default()).unwrap();
    let results = WafScenarios::test_xss_blocking(&engine);

    let blocked_count = results.iter().filter(|(_, blocked)| *blocked).count();
    println!("XSS: {}/{} blocked", blocked_count, results.len());

    for (payload, blocked) in &results {
        assert!(blocked, "XSS should be blocked: {}", payload);
    }
}

/// Test WAF blocks path traversal
#[test]
fn test_waf_blocks_path_traversal() {
    let engine = WafEngine::new(WafConfig::default()).unwrap();
    let results = WafScenarios::test_path_traversal_blocking(&engine);

    for (payload, blocked) in &results {
        assert!(blocked, "Path traversal should be blocked: {}", payload);
    }
}

/// Test WAF allows legitimate traffic
#[test]
fn test_waf_allows_legitimate_traffic() {
    let engine = WafEngine::new(WafConfig::default()).unwrap();
    let results = WafScenarios::test_legitimate_traffic(&engine);

    for (payload, allowed) in &results {
        assert!(allowed, "Legitimate traffic should be allowed: {}", payload);
    }
}

/// Test rate limiter enforces limits
#[test]
fn test_rate_limiter_enforces_limit() {
    let (allowed, blocked) = RateLimitScenarios::test_basic_limiting(100);

    assert_eq!(allowed, 100, "Should allow exactly 100 requests");
    assert_eq!(blocked, 10, "Should block 10 excess requests");
}

/// Test rate limiter allows burst
#[test]
fn test_rate_limiter_allows_burst() {
    let allowed = RateLimitScenarios::test_burst_handling(100, 20);

    assert_eq!(allowed, 120, "Should allow limit + burst = 120 requests");
}

/// Test rate limiter isolates clients
#[test]
fn test_rate_limiter_client_isolation() {
    let isolated = RateLimitScenarios::test_client_isolation(10);

    assert!(isolated, "Rate limits should be isolated per client");
}

/// Test WAF performance is under 1ms
#[test]
fn test_waf_performance_under_1ms() {
    let (min, avg, max) = PerformanceScenarios::benchmark_waf_scan(1000);

    println!("WAF performance: min={}us, avg={}us, max={}us", min, avg, max);

    assert!(avg < 1000, "WAF average scan should be under 1ms, got {}us", avg);
}

/// Test rate limiter performance is under 10us
#[test]
fn test_rate_limiter_performance_under_10us() {
    let (min, avg, max) = PerformanceScenarios::benchmark_rate_limiter(10000);

    println!("Rate limiter performance: min={}ns, avg={}ns, max={}ns", min, avg, max);

    assert!(avg < 10000, "Rate limiter average check should be under 10us, got {}ns", avg);
}

/// Test random JSON payload handling
#[test]
fn test_random_payload_handling() {
    let engine = WafEngine::new(WafConfig::default()).unwrap();

    // Generate 100 random payloads
    for _ in 0..100 {
        let payload = PayloadGenerator::random_json(3, 5);
        let payload_str = payload.to_string();

        // Should not panic
        let result = engine.scan(&payload_str);

        // Random JSON should generally be allowed (no attack patterns)
        assert!(result.allowed, "Random JSON should be allowed: {}", payload_str);
    }
}

/// Test route fixture creation
#[test]
fn test_route_fixture() {
    let route = RouteFixture::new("/api/users", "http://backend:8080")
        .with_method("POST")
        .with_transform(r#"
            let data = parse_json(input);
            data["timestamp"] = timestamp_ms();
            output = to_json(data);
        "#);

    assert_eq!(route.path, "/api/users");
    assert_eq!(route.method, "POST");
    assert!(route.upstream.contains("backend"));
    assert!(route.transform_script.is_some());
}

/// Summary test that runs all scenarios
#[test]
fn test_full_security_suite() {
    let engine = WafEngine::new(WafConfig::default()).unwrap();

    // SQL Injection
    let sql_results = WafScenarios::test_sql_injection_blocking(&engine);
    let sql_blocked = sql_results.iter().filter(|(_, b)| *b).count();

    // XSS
    let xss_results = WafScenarios::test_xss_blocking(&engine);
    let xss_blocked = xss_results.iter().filter(|(_, b)| *b).count();

    // Path Traversal
    let path_results = WafScenarios::test_path_traversal_blocking(&engine);
    let path_blocked = path_results.iter().filter(|(_, b)| *b).count();

    // Legitimate
    let legit_results = WafScenarios::test_legitimate_traffic(&engine);
    let legit_allowed = legit_results.iter().filter(|(_, a)| *a).count();

    println!("\n=== Security Suite Results ===");
    println!("SQL Injection: {}/{} blocked", sql_blocked, sql_results.len());
    println!("XSS: {}/{} blocked", xss_blocked, xss_results.len());
    println!("Path Traversal: {}/{} blocked", path_blocked, path_results.len());
    println!("Legitimate: {}/{} allowed", legit_allowed, legit_results.len());
    println!("==============================\n");

    // Assertions
    assert_eq!(sql_blocked, sql_results.len(), "All SQL injections must be blocked");
    assert_eq!(xss_blocked, xss_results.len(), "All XSS must be blocked");
    assert_eq!(path_blocked, path_results.len(), "All path traversals must be blocked");
    assert_eq!(legit_allowed, legit_results.len(), "All legitimate traffic must be allowed");
}

//! Health Check Module
//!
//! Provides deep health checks with dependency status for Kubernetes probes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

/// Health check result for a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(rename = "responseTimeMs")]
    pub response_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Overall health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    #[serde(rename = "uptimeSeconds")]
    pub uptime_seconds: u64,
    pub components: HashMap<String, ComponentHealth>,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Timeout for each health check
    pub timeout: Duration,
    /// Whether to include details in response
    pub include_details: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            include_details: true,
        }
    }
}

/// Health checker that aggregates component health
pub struct HealthChecker {
    config: HealthCheckConfig,
    start_time: Instant,
    version: String,
    checks: RwLock<Vec<Box<dyn HealthCheck + Send + Sync>>>,
}

use futures::future::BoxFuture;

/// Trait for implementing health checks
pub trait HealthCheck: Send + Sync {
    /// Name of the component being checked
    fn name(&self) -> &str;

    /// Perform the health check
    fn check(&self) -> BoxFuture<'static, ComponentHealth>;
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks: RwLock::new(Vec::new()),
        }
    }

    /// Register a health check
    pub async fn register(&self, check: Box<dyn HealthCheck + Send + Sync>) {
        let mut checks = self.checks.write().await;
        checks.push(check);
    }

    /// Perform liveness check (is the process alive?)
    pub fn liveness(&self) -> HealthResponse {
        HealthResponse {
            status: HealthStatus::Healthy,
            version: self.version.clone(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            components: HashMap::new(),
        }
    }

    /// Perform readiness check (can handle traffic?)
    pub async fn readiness(&self) -> HealthResponse {
        let checks = self.checks.read().await;
        let mut components = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        for check in checks.iter() {
            // Start time check
            
            let result = tokio::time::timeout(self.config.timeout, check.check()).await;

            let health = match result {
                Ok(h) => h,
                Err(_) => {
                    warn!(component = %check.name(), "Health check timed out");
                    ComponentHealth {
                        name: check.name().to_string(),
                        status: HealthStatus::Unhealthy,
                        message: Some("Health check timed out".to_string()),
                        response_time_ms: self.config.timeout.as_millis() as u64,
                        details: None,
                    }
                }
            };

            // Update overall status
            match health.status {
                HealthStatus::Unhealthy => overall_status = HealthStatus::Unhealthy,
                HealthStatus::Degraded if overall_status == HealthStatus::Healthy => {
                    overall_status = HealthStatus::Degraded;
                }
                _ => {}
            }

            debug!(
                component = %health.name,
                status = ?health.status,
                time_ms = %health.response_time_ms,
                "Health check completed"
            );

            components.insert(health.name.clone(), health);
        }

        HealthResponse {
            status: overall_status,
            version: self.version.clone(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            components,
        }
    }
}

// DatabaseHealthCheck impl
pub struct DatabaseHealthCheck {
    name: String,
    check_fn: Arc<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>> + Send + Sync>,
}

impl DatabaseHealthCheck {
    pub fn new<F, Fut>(name: &str, check_fn: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = bool> + Send + 'static,
    {
        Self {
            name: name.to_string(),
            check_fn: Arc::new(move || Box::pin(check_fn())),
        }
    }
}

impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> BoxFuture<'static, ComponentHealth> {
        let name = self.name.clone();
        let check_fn = self.check_fn.clone();
        
        Box::pin(async move {
            let start = Instant::now();
            let is_healthy = (check_fn)().await;
            let elapsed = start.elapsed().as_millis() as u64;

            ComponentHealth {
                name,
                status: if is_healthy { HealthStatus::Healthy } else { HealthStatus::Unhealthy },
                message: if is_healthy { None } else { Some("Connection failed".to_string()) },
                response_time_ms: elapsed,
                details: None,
            }
        })
    }
}

/// Simple ping health check
pub struct PingHealthCheck {
    name: String,
}

impl PingHealthCheck {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

impl HealthCheck for PingHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> BoxFuture<'static, ComponentHealth> {
        let name = self.name.clone();
        Box::pin(async move {
            ComponentHealth {
                name,
                status: HealthStatus::Healthy,
                message: None,
                response_time_ms: 0,
                details: None,
            }
        })
    }
}

/// Memory health check
pub struct MemoryHealthCheck {
    /// Threshold in bytes for degraded status
    degraded_threshold: usize,
    /// Threshold in bytes for unhealthy status
    unhealthy_threshold: usize,
}

impl MemoryHealthCheck {
    pub fn new(degraded_mb: usize, unhealthy_mb: usize) -> Self {
        Self {
            degraded_threshold: degraded_mb * 1024 * 1024,
            unhealthy_threshold: unhealthy_mb * 1024 * 1024,
        }
    }
}

impl HealthCheck for MemoryHealthCheck {
    fn name(&self) -> &str {
        "memory"
    }

    fn check(&self) -> BoxFuture<'static, ComponentHealth> {
        let degraded = self.degraded_threshold;
        let unhealthy = self.unhealthy_threshold;
        
        Box::pin(async move {
            // Simple approximation - in production, use a proper memory stats library
            let estimated_usage = 100 * 1024 * 1024; // 100MB placeholder
    
            let status = if estimated_usage >= unhealthy {
                HealthStatus::Unhealthy
            } else if estimated_usage >= degraded {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            };
    
            ComponentHealth {
                name: "memory".to_string(),
                status,
                message: None,
                response_time_ms: 0,
                details: Some(serde_json::json!({
                    "estimated_bytes": estimated_usage,
                    "degraded_threshold": degraded,
                    "unhealthy_threshold": unhealthy,
                })),
            }
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_liveness() {
        let checker = HealthChecker::new(HealthCheckConfig::default());
        let response = checker.liveness();
        
        assert_eq!(response.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_readiness_empty() {
        let checker = HealthChecker::new(HealthCheckConfig::default());
        let response = checker.readiness().await;
        
        assert_eq!(response.status, HealthStatus::Healthy);
        assert!(response.components.is_empty());
    }

    #[tokio::test]
    async fn test_ping_health_check() {
        let check = PingHealthCheck::new("test");
        let result = check.check().await;
        
        assert_eq!(result.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status_methods() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(HealthStatus::Healthy.is_ready());
        assert!(HealthStatus::Degraded.is_ready());
        assert!(!HealthStatus::Unhealthy.is_ready());
    }
}

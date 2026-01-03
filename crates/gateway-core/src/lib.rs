//! # Gateway Core
//!
//! Core HTTP gateway logic providing routing, body handling, and service traits.
//! This crate is intentionally isolated from database dependencies for faster
//! compilation and easier testing.

pub mod body;
pub mod config;
pub mod auth;
pub mod error;
pub mod executor;
pub mod handler;
pub mod router;
pub mod transform;
pub mod observability;
pub mod health;
pub mod circuit_breaker;

pub use config::{Route, RouterMap};
pub use auth::{User, Role, ApiKey};
pub use error::GatewayError;
pub use executor::TokioExecutor;
pub use handler::handle_request;
pub use router::{build_router_map, match_route};
pub use transform::{RhaiTransformer, TransformError, TransformResult, simulate, validate_script};
pub use observability::{MetricsCollector, MetricsSnapshot, RequestMetrics, TraceContext};
pub use health::{
    HealthChecker, HealthCheckConfig, HealthStatus, HealthResponse, HealthCheck,
    DatabaseHealthCheck, PingHealthCheck, MemoryHealthCheck
};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerRegistry, State as CircuitState};

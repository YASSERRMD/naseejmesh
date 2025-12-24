//! # Naseej Security
//!
//! Enterprise security layer for NaseejMesh providing:
//! - Web Application Firewall (WAF)
//! - JWT/OIDC Authentication
//! - Distributed Rate Limiting
//! - Request/Response validation

pub mod waf;
pub mod auth;
pub mod key_manager;
pub mod rate_limit;
pub mod metering;

pub use waf::{WafEngine, WafConfig, WafResult};
pub use auth::{JwtValidator, JwtIssuer, AuthConfig, Claims};
pub use key_manager::{KeyManager, KeyManagerError};
pub use rate_limit::{RateLimiter, RateLimitConfig, RateLimitResult};
pub use metering::{Meter, UsageEvent};

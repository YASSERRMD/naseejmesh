//! Error types for the API Gateway with fail-fast classification.
//!
//! Each error variant is designed to provide specific, actionable information
//! to aid in debugging and monitoring.


use thiserror::Error;

/// Gateway-specific errors with detailed classification
#[derive(Debug, Error)]
pub enum GatewayError {
    /// Route not found for the requested path
    #[error("Route not found: {path}")]
    RouteNotFound { path: String },

    /// Method not allowed for this route
    #[error("Method {method} not allowed for path: {path}")]
    MethodNotAllowed { method: String, path: String },

    /// Request payload exceeds configured limit
    #[error("Payload too large: received {size} bytes, limit is {limit} bytes")]
    PayloadTooLarge { size: u64, limit: u64 },

    /// Failed to connect to upstream service
    #[error("Upstream connection failed for {upstream}: {reason}")]
    UpstreamConnectionFailed { upstream: String, reason: String },

    /// Upstream service returned an error
    #[error("Upstream error from {upstream}: {status_code}")]
    UpstreamError { upstream: String, status_code: u16 },

    /// Request timed out waiting for upstream
    #[error("Request timeout after {timeout_ms}ms for upstream: {upstream}")]
    RequestTimeout { upstream: String, timeout_ms: u64 },

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Internal server error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Body read error
    #[error("Failed to read request body: {0}")]
    BodyReadError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl GatewayError {
    /// Get the appropriate HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            GatewayError::RouteNotFound { .. } => 404,
            GatewayError::MethodNotAllowed { .. } => 405,
            GatewayError::PayloadTooLarge { .. } => 413,
            GatewayError::UpstreamConnectionFailed { .. } => 502,
            GatewayError::UpstreamError { status_code, .. } => *status_code,
            GatewayError::RequestTimeout { .. } => 504,
            GatewayError::ConfigError(_) => 500,
            GatewayError::DatabaseError(_) => 500,
            GatewayError::InternalError(_) => 500,
            GatewayError::BodyReadError(_) => 400,
            GatewayError::SerializationError(_) => 400,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            GatewayError::UpstreamConnectionFailed { .. } => true,
            GatewayError::RequestTimeout { .. } => true,
            GatewayError::UpstreamError { status_code, .. } => *status_code >= 500,
            _ => false,
        }
    }

    /// Get the error category for metrics/logging
    pub fn category(&self) -> &'static str {
        match self {
            GatewayError::RouteNotFound { .. } => "routing",
            GatewayError::MethodNotAllowed { .. } => "routing",
            GatewayError::PayloadTooLarge { .. } => "client_error",
            GatewayError::UpstreamConnectionFailed { .. } => "upstream",
            GatewayError::UpstreamError { .. } => "upstream",
            GatewayError::RequestTimeout { .. } => "upstream",
            GatewayError::ConfigError(_) => "config",
            GatewayError::DatabaseError(_) => "database",
            GatewayError::InternalError(_) => "internal",
            GatewayError::BodyReadError(_) => "client_error",
            GatewayError::SerializationError(_) => "client_error",
        }
    }
}

/// Wrapper for upstream errors from hyper/http operations
impl From<hyper::Error> for GatewayError {
    fn from(err: hyper::Error) -> Self {
        GatewayError::InternalError(err.to_string())
    }
}

impl From<std::io::Error> for GatewayError {
    fn from(err: std::io::Error) -> Self {
        GatewayError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for GatewayError {
    fn from(err: serde_json::Error) -> Self {
        GatewayError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_codes() {
        assert_eq!(
            GatewayError::RouteNotFound {
                path: "/test".into()
            }
            .status_code(),
            404
        );
        assert_eq!(
            GatewayError::PayloadTooLarge { size: 100, limit: 50 }.status_code(),
            413
        );
        assert_eq!(
            GatewayError::RequestTimeout {
                upstream: "test".into(),
                timeout_ms: 1000
            }
            .status_code(),
            504
        );
    }

    #[test]
    fn test_retryable() {
        assert!(GatewayError::UpstreamConnectionFailed {
            upstream: "test".into(),
            reason: "refused".into()
        }
        .is_retryable());

        assert!(!GatewayError::RouteNotFound {
            path: "/test".into()
        }
        .is_retryable());
    }
}

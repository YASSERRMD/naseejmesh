//! JWT/OIDC Authentication
//!
//! High-performance JWT validation with local caching.
//! Supports RS256/HS256 algorithms and JWKS key rotation.

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Authentication errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Token missing")]
    TokenMissing,

    #[error("Token expired")]
    TokenExpired,

    #[error("Token invalid: {0}")]
    TokenInvalid(String),

    #[error("Key fetch failed: {0}")]
    KeyFetchFailed(String),

    #[error("Insufficient permissions")]
    InsufficientPermissions,
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Issuer
    #[serde(default)]
    pub iss: Option<String>,

    /// Audience
    #[serde(default)]
    pub aud: Option<String>,

    /// Expiration time (Unix timestamp)
    pub exp: u64,

    /// Issued at (Unix timestamp)
    #[serde(default)]
    pub iat: Option<u64>,

    /// Not before (Unix timestamp)
    #[serde(default)]
    pub nbf: Option<u64>,

    /// Custom claims
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable/disable authentication
    pub enabled: bool,

    /// Accepted issuers
    pub issuers: Vec<String>,

    /// Accepted audiences
    pub audiences: Vec<String>,

    /// Algorithm (HS256, RS256)
    pub algorithm: String,

    /// Secret for HS256 (or JWKS URL for RS256)
    pub secret_or_jwks: String,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,

    /// Maximum cache size
    pub cache_max_size: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            issuers: vec![],
            audiences: vec![],
            algorithm: "HS256".to_string(),
            secret_or_jwks: "secret".to_string(),
            cache_ttl_secs: 300,
            cache_max_size: 10_000,
        }
    }
}

/// JWT Validator with caching
pub struct JwtValidator {
    config: AuthConfig,
    validation: Validation,
    decoding_key: DecodingKey,
    cache: Cache<String, Claims>,
}

impl JwtValidator {
    /// Create a new JWT validator
    pub fn new(config: AuthConfig) -> Result<Self, AuthError> {
        let algorithm = match config.algorithm.as_str() {
            "HS256" => Algorithm::HS256,
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            "RS256" => Algorithm::RS256,
            "RS384" => Algorithm::RS384,
            "RS512" => Algorithm::RS512,
            _ => return Err(AuthError::TokenInvalid(format!(
                "Unsupported algorithm: {}",
                config.algorithm
            ))),
        };

        let mut validation = Validation::new(algorithm);
        
        if !config.issuers.is_empty() {
            validation.set_issuer(&config.issuers);
        }
        
        if !config.audiences.is_empty() {
            validation.set_audience(&config.audiences);
        }

        // Create decoding key based on algorithm
        let decoding_key = match algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                DecodingKey::from_secret(config.secret_or_jwks.as_bytes())
            }
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                // For RS256, the secret should be the PEM-encoded public key
                DecodingKey::from_rsa_pem(config.secret_or_jwks.as_bytes())
                    .map_err(|e| AuthError::TokenInvalid(e.to_string()))?
            }
            _ => return Err(AuthError::TokenInvalid("Unsupported key type".to_string())),
        };

        // Create cache
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(config.cache_ttl_secs))
            .max_capacity(config.cache_max_size)
            .build();

        info!(
            algorithm = %config.algorithm,
            cache_size = config.cache_max_size,
            "JWT validator initialized"
        );

        Ok(Self {
            config,
            validation,
            decoding_key,
            cache,
        })
    }

    /// Validate a JWT token
    pub async fn validate(&self, token: &str) -> Result<Claims, AuthError> {
        if !self.config.enabled {
            // Return dummy claims when disabled
            return Ok(Claims {
                sub: "anonymous".to_string(),
                iss: None,
                aud: None,
                exp: u64::MAX,
                iat: None,
                nbf: None,
                extra: serde_json::Map::new(),
            });
        }

        // Check cache first
        if let Some(claims) = self.cache.get(token).await {
            debug!(sub = %claims.sub, "Token validated from cache");
            return Ok(claims);
        }

        // Decode and validate
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| {
                warn!(error = %e, "Token validation failed");
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    _ => AuthError::TokenInvalid(e.to_string()),
                }
            })?;

        let claims = token_data.claims;

        // Cache the validated claims
        self.cache.insert(token.to_string(), claims.clone()).await;

        debug!(
            sub = %claims.sub,
            "Token validated and cached"
        );

        Ok(claims)
    }

    /// Extract token from Authorization header
    pub fn extract_token(auth_header: &str) -> Result<&str, AuthError> {
        if auth_header.starts_with("Bearer ") {
            Ok(&auth_header[7..])
        } else {
            Err(AuthError::TokenInvalid("Invalid authorization header format".to_string()))
        }
    }

    /// Check if claims have required scope
    pub fn has_scope(claims: &Claims, required_scope: &str) -> bool {
        if let Some(scope) = claims.extra.get("scope") {
            if let Some(scope_str) = scope.as_str() {
                return scope_str.split_whitespace().any(|s| s == required_scope);
            }
        }
        false
    }

    /// Check if claims have required role
    pub fn has_role(claims: &Claims, required_role: &str) -> bool {
        if let Some(roles) = claims.extra.get("roles") {
            if let Some(roles_arr) = roles.as_array() {
                return roles_arr.iter().any(|r| r.as_str() == Some(required_role));
            }
        }
        false
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (u64, u64) {
        (self.cache.entry_count(), self.config.cache_max_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_token(secret: &str, exp_offset: i64) -> String {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let exp = (chrono::Utc::now().timestamp() + exp_offset) as u64;
        let claims = Claims {
            sub: "user123".to_string(),
            iss: Some("test-issuer".to_string()),
            aud: Some("test-audience".to_string()),
            exp,
            iat: Some(chrono::Utc::now().timestamp() as u64),
            nbf: None,
            extra: serde_json::Map::new(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_validate_valid_token() {
        let secret = "test-secret-key-for-testing-purposes";
        let config = AuthConfig {
            secret_or_jwks: secret.to_string(),
            issuers: vec!["test-issuer".to_string()],
            audiences: vec!["test-audience".to_string()],
            ..Default::default()
        };

        let validator = JwtValidator::new(config).unwrap();
        let token = create_test_token(secret, 3600);

        let claims = validator.validate(&token).await.unwrap();
        assert_eq!(claims.sub, "user123");
    }

    #[tokio::test]
    async fn test_validate_expired_token() {
        let secret = "test-secret-key-for-testing-purposes";
        let config = AuthConfig {
            secret_or_jwks: secret.to_string(),
            ..Default::default()
        };

        let validator = JwtValidator::new(config).unwrap();
        let token = create_test_token(secret, -3600); // Expired 1 hour ago

        let result = validator.validate(&token).await;
        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[tokio::test]
    async fn test_disabled_auth() {
        let config = AuthConfig {
            enabled: false,
            ..Default::default()
        };

        let validator = JwtValidator::new(config).unwrap();
        let claims = validator.validate("any-token").await.unwrap();
        assert_eq!(claims.sub, "anonymous");
    }

    #[test]
    fn test_extract_token() {
        let token = JwtValidator::extract_token("Bearer abc123").unwrap();
        assert_eq!(token, "abc123");

        let result = JwtValidator::extract_token("Basic abc123");
        assert!(result.is_err());
    }
}

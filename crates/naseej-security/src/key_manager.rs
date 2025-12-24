//! Key Manager for secure credential handling.
//!
//! Handles password hashing (bcrypt) and API key generation/verification (SHA-256).

use bcrypt::{hash, verify, DEFAULT_COST};
use sha2::{Digest, Sha256};
use rand::{distributions::Alphanumeric, Rng};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyManagerError {
    #[error("Hashing failed: {0}")]
    HashError(String),
}

/// Manages secure hashing and verification of credentials.
pub struct KeyManager;

impl KeyManager {
    /// Hash a password using bcrypt.
    pub fn hash_password(password: &str) -> Result<String, KeyManagerError> {
        hash(password, DEFAULT_COST).map_err(|e| KeyManagerError::HashError(e.to_string()))
    }

    /// Verify a password against a bcrypt hash.
    pub fn verify_password(password: &str, hash: &str) -> bool {
        verify(password, hash).unwrap_or(false)
    }

    /// Generate a secure API Key.
    /// Returns (raw_key_for_user, hash_for_db, prefix).
    /// Format: "nas_sk_<24_random_chars>"
    pub fn generate_api_key() -> (String, String, String) {
        let random_part: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect();
        
        let raw_key = format!("nas_sk_{}", random_part);
        let prefix = "nas_sk_".to_string();
        
        let hash = Self::hash_api_key(&raw_key);
        
        (raw_key, hash, prefix)
    }

    /// Hash an API key for storage using SHA-256 (fast & secure).
    pub fn hash_api_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify an API key against its SHA-256 hash.
    pub fn verify_api_key(raw_key: &str, stored_hash: &str) -> bool {
        let hash = Self::hash_api_key(raw_key);
        // Constant-time comparison to prevent timing attacks
        subtle::ConstantTimeEq::ct_eq(hash.as_bytes(), stored_hash.as_bytes()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "supersecret";
        let hash = KeyManager::hash_password(password).unwrap();
        
        assert!(KeyManager::verify_password(password, &hash));
        assert!(!KeyManager::verify_password("wrongpass", &hash));
    }

    #[test]
    fn test_api_key_generation() {
        let (raw, hash, prefix) = KeyManager::generate_api_key();
        
        assert!(raw.starts_with("nas_sk_"));
        assert_eq!(prefix, "nas_sk_");
        assert_ne!(raw, hash);
        
        assert!(KeyManager::verify_api_key(&raw, &hash));
        assert!(!KeyManager::verify_api_key("wrongkey", &hash));
    }
}

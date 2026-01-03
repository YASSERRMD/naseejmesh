use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::serde_utils::deserialize_id;

/// Represents a system user for the admin console.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct User {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub username: String,
    // Store as bcrypt hash
    pub password_hash: String,
    pub roles: Vec<String>,
    pub active: bool,
    pub created_at: String,
}

/// Represents a role with a set of permissions.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Role {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub permissions: Vec<String>,
    pub created_at: String,
}

/// Represents an API Key for outbound gateway access.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ApiKey {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    // Store as SHA-256 hash
    pub key_hash: String,
    // First 8 chars for display/identification
    pub prefix: String, 
    pub owner_id: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

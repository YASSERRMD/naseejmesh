use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
    pub created_at: DateTime<Utc>,
}

/// Represents a role with a set of permissions.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Role {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
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
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Custom deserializer to handle SurrealDB Record IDs which can be strings or objects
pub fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Object(map) => {
            let tb = map.get("tb").and_then(|v| v.as_str()).unwrap_or("unknown");
            let id = map.get("id").map(|v| {
                if let Some(s) = v.as_str() {
                    s.to_string()
                } else {
                    v.to_string()
                }
            }).unwrap_or_else(|| "unknown".to_string());
            Ok(format!("{}:{}", tb, id))
        }
        _ => Err(serde::de::Error::custom("Invalid ID format: expected string or object")),
    }
}

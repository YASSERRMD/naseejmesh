//! Auth schema and CRUD operations for SurrealDB.
//!
//! Handles persistence for Users, Roles, and API Keys.

use gateway_core::auth::{User, Role, ApiKey};
use surrealdb::Connection;
use surrealdb::Surreal;
use chrono::Utc;
use crate::error::ConfigError;

const USERS_TABLE: &str = "users";
const ROLES_TABLE: &str = "roles";
const KEYS_TABLE: &str = "api_keys";

// ... imports

pub async fn create_user<C: Connection>(db: &Surreal<C>, mut user: User) -> Result<User, ConfigError> {
    user.created_at = Utc::now();
    
    let created: Option<User> = db
        .create((USERS_TABLE, &user.id))
        .content(user)
        .await?;

    created.ok_or_else(|| ConfigError::Database("Failed to create user".to_string()))
}

pub async fn get_user<C: Connection>(db: &Surreal<C>, id: &str) -> Result<Option<User>, ConfigError> {
    let user: Option<User> = db.select((USERS_TABLE, id)).await?;
    Ok(user)
}

pub async fn get_user_by_username<C: Connection>(db: &Surreal<C>, username: &str) -> Result<Option<User>, ConfigError> {
    let mut result = db
        .query("SELECT * FROM type::table($table) WHERE username = $username")
        .bind(("table", USERS_TABLE))
        .bind(("username", username.to_string()))
        .await?;
    
    let user: Option<User> = result.take(0)?;
    Ok(user)
}

pub async fn list_users<C: Connection>(db: &Surreal<C>) -> Result<Vec<User>, ConfigError> {
    let users: Vec<User> = db.select(USERS_TABLE).await?;
    Ok(users)
}

// ============================================================================
// Role Operations
// ============================================================================

pub async fn create_role<C: Connection>(db: &Surreal<C>, mut role: Role) -> Result<Role, ConfigError> {
    role.created_at = Utc::now();

    let created: Option<Role> = db
        .create((ROLES_TABLE, &role.id))
        .content(role)
        .await?;

    created.ok_or_else(|| ConfigError::Database("Failed to create role".to_string()))
}

pub async fn get_role<C: Connection>(db: &Surreal<C>, id: &str) -> Result<Option<Role>, ConfigError> {
    let role: Option<Role> = db.select((ROLES_TABLE, id)).await?;
    Ok(role)
}

pub async fn list_roles<C: Connection>(db: &Surreal<C>) -> Result<Vec<Role>, ConfigError> {
    let roles: Vec<Role> = db.select(ROLES_TABLE).await?;
    Ok(roles)
}

// ============================================================================
// API Key Operations
// ============================================================================

pub async fn create_api_key<C: Connection>(db: &Surreal<C>, mut key: ApiKey) -> Result<ApiKey, ConfigError> {
    key.created_at = Utc::now();

    let created: Option<ApiKey> = db
        .create((KEYS_TABLE, &key.id))
        .content(key)
        .await?;

    created.ok_or_else(|| ConfigError::Database("Failed to create API key".to_string()))
}

pub async fn get_api_key<C: Connection>(db: &Surreal<C>, id: &str) -> Result<Option<ApiKey>, ConfigError> {
    let key: Option<ApiKey> = db.select((KEYS_TABLE, id)).await?;
    Ok(key)
}

pub async fn list_api_keys<C: Connection>(db: &Surreal<C>) -> Result<Vec<ApiKey>, ConfigError> {
    let keys: Vec<ApiKey> = db.select(KEYS_TABLE).await?;
    Ok(keys)
}

pub async fn delete_api_key<C: Connection>(db: &Surreal<C>, id: &str) -> Result<(), ConfigError> {
    let deleted: Option<ApiKey> = db.delete((KEYS_TABLE, id)).await?;
    if deleted.is_none() {
        return Err(ConfigError::Database(format!("API Key {} not found", id)));
    }
    Ok(())
}

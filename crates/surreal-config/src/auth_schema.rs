//! Auth schema and CRUD operations for SurrealDB.
//!
//! Handles persistence for Users, Roles, and API Keys.

use gateway_core::auth::{User, Role, ApiKey};
use surrealdb::Connection;
use surrealdb::Surreal;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::error::ConfigError;

const USERS_TABLE: &str = "users";
const ROLES_TABLE: &str = "roles";
const KEYS_TABLE: &str = "api_keys";

// ============================================================================
// Internal Database Structures (SurrealDB v2 compatible)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct DbUser {
    id: surrealdb::sql::Thing,
    username: String,
    password_hash: String,
    roles: Vec<String>,
    active: bool,
    created_at: String,
}

impl From<DbUser> for User {
    fn from(db: DbUser) -> Self {
        User {
            id: db.id.to_string(),
            username: db.username,
            password_hash: db.password_hash,
            roles: db.roles,
            active: db.active,
            created_at: db.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DbRole {
    id: surrealdb::sql::Thing,
    name: String,
    permissions: Vec<String>,
    created_at: String,
}

impl From<DbRole> for Role {
    fn from(db: DbRole) -> Self {
        Role {
            id: db.id.to_string(),
            name: db.name,
            permissions: db.permissions,
            created_at: db.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DbApiKey {
    id: surrealdb::sql::Thing,
    name: String,
    key_hash: String,
    prefix: String,
    owner_id: String,
    scopes: Vec<String>,
    expires_at: Option<String>,
    created_at: String,
    last_used_at: Option<String>,
}

impl From<DbApiKey> for ApiKey {
    fn from(db: DbApiKey) -> Self {
        ApiKey {
            id: db.id.to_string(),
            name: db.name,
            key_hash: db.key_hash,
            prefix: db.prefix,
            owner_id: db.owner_id,
            scopes: db.scopes,
            expires_at: db.expires_at,
            created_at: db.created_at,
            last_used_at: db.last_used_at,
        }
    }
}

// ============================================================================
// User Operations
// ============================================================================

pub async fn create_user<C: Connection>(db: &Surreal<C>, mut user: User) -> Result<User, ConfigError> {
    user.created_at = Utc::now().to_rfc3339();
    
    let mut result = db.query("CREATE type::thing('users', $id) SET username = $user, password_hash = $pass, roles = $roles, active = $active, created_at = $now")
        .bind(("id", user.id.clone()))
        .bind(("user", user.username))
        .bind(("pass", user.password_hash))
        .bind(("roles", user.roles))
        .bind(("active", user.active))
        .bind(("now", user.created_at))
        .await?;
    
    let created: Option<DbUser> = result.take(0)?;
    created.map(User::from).ok_or_else(|| ConfigError::Database("Failed to create user".to_string()))
}

pub async fn get_user<C: Connection>(db: &Surreal<C>, id: &str) -> Result<Option<User>, ConfigError> {
    let user: Option<DbUser> = db.select((USERS_TABLE, id)).await?;
    Ok(user.map(User::from))
}

pub async fn get_user_by_username<C: Connection>(db: &Surreal<C>, username: &str) -> Result<Option<User>, ConfigError> {
    let mut result = db
        .query("SELECT * FROM users WHERE username = $username")
        .bind(("username", username.to_string()))
        .await?;
    
    let user: Option<DbUser> = result.take(0)?;
    Ok(user.map(User::from))
}

pub async fn list_users<C: Connection>(db: &Surreal<C>) -> Result<Vec<User>, ConfigError> {
    let users: Vec<DbUser> = db.select(USERS_TABLE).await?;
    Ok(users.into_iter().map(User::from).collect())
}

// ============================================================================
// Role Operations
// ============================================================================

pub async fn create_role<C: Connection>(db: &Surreal<C>, mut role: Role) -> Result<Role, ConfigError> {
    role.created_at = Utc::now().to_rfc3339();

    let mut result = db.query("CREATE type::thing('roles', $id) SET name = $name, permissions = $perm, created_at = $now")
        .bind(("id", role.id.clone()))
        .bind(("name", role.name))
        .bind(("perm", role.permissions))
        .bind(("now", role.created_at))
        .await?;

    let created: Option<DbRole> = result.take(0)?;
    created.map(Role::from).ok_or_else(|| ConfigError::Database("Failed to create role".to_string()))
}

pub async fn get_role<C: Connection>(db: &Surreal<C>, id: &str) -> Result<Option<Role>, ConfigError> {
    let role: Option<DbRole> = db.select((ROLES_TABLE, id)).await?;
    Ok(role.map(Role::from))
}

pub async fn list_roles<C: Connection>(db: &Surreal<C>) -> Result<Vec<Role>, ConfigError> {
    let roles: Vec<DbRole> = db.select(ROLES_TABLE).await?;
    Ok(roles.into_iter().map(Role::from).collect())
}

// ============================================================================
// API Key Operations
// ============================================================================

pub async fn create_api_key<C: Connection>(db: &Surreal<C>, mut key: ApiKey) -> Result<ApiKey, ConfigError> {
    key.created_at = Utc::now().to_rfc3339();

    let mut result = db.query("CREATE type::thing('api_keys', $id) SET name = $name, key_hash = $hash, prefix = $pref, owner_id = $owner, scopes = $scopes, expires_at = $exp, created_at = $now, last_used_at = $used")
        .bind(("id", key.id.clone()))
        .bind(("name", key.name))
        .bind(("hash", key.key_hash))
        .bind(("pref", key.prefix))
        .bind(("owner", key.owner_id))
        .bind(("scopes", key.scopes))
        .bind(("exp", key.expires_at))
        .bind(("now", key.created_at))
        .bind(("used", key.last_used_at))
        .await?;

    let created: Option<DbApiKey> = result.take(0)?;
    created.map(ApiKey::from).ok_or_else(|| ConfigError::Database("Failed to create API key".to_string()))
}

pub async fn get_api_key<C: Connection>(db: &Surreal<C>, id: &str) -> Result<Option<ApiKey>, ConfigError> {
    let key: Option<DbApiKey> = db.select((KEYS_TABLE, id)).await?;
    Ok(key.map(ApiKey::from))
}

pub async fn list_api_keys<C: Connection>(db: &Surreal<C>) -> Result<Vec<ApiKey>, ConfigError> {
    let keys: Vec<DbApiKey> = db.select(KEYS_TABLE).await?;
    Ok(keys.into_iter().map(ApiKey::from).collect())
}

pub async fn delete_api_key<C: Connection>(db: &Surreal<C>, id: &str) -> Result<(), ConfigError> {
    let deleted: Option<DbApiKey> = db.delete((KEYS_TABLE, id)).await?;
    if deleted.is_none() {
        return Err(ConfigError::Database(format!("API Key {} not found", id)));
    }
    Ok(())
}

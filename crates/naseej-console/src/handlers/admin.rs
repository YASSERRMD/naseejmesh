use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::state::AppState;
use gateway_core::auth::{User, Role, ApiKey};
use surreal_config::auth_schema::{
    create_user, list_users,
    create_role, list_roles,
    create_api_key, list_api_keys, delete_api_key
};
use naseej_security::KeyManager;

// ===================================
// Types
// ===================================

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    pub owner_id: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub key: ApiKey,
    pub raw_key: String, // Only returned on creation
}

// ===================================
// User Handlers
// ===================================

pub async fn list_users_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let users = list_users(&state.db).await.map_err(|e| {
        error!("List users failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(users))
}

pub async fn create_user_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, (StatusCode, String)> {
    let password_hash = KeyManager::hash_password(&req.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = User {
        id: Uuid::new_v4().to_string(),
        username: req.username,
        password_hash,
        roles: req.roles,
        active: true,
        created_at: chrono::Utc::now(),
    };

    let created = create_user(&state.db, user).await.map_err(|e| {
        error!("Create user failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(created))
}

// ===================================
// Role Handlers
// ===================================

pub async fn list_roles_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Role>>, (StatusCode, String)> {
    let roles = list_roles(&state.db).await.map_err(|e| {
        error!("List roles failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(roles))
}

pub async fn create_role_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRoleRequest>,
) -> Result<Json<Role>, (StatusCode, String)> {
    let role = Role {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        permissions: req.permissions,
        created_at: chrono::Utc::now(),
    };

    let created = create_role(&state.db, role).await.map_err(|e| {
        error!("Create role failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(created))
}

// ===================================
// API Key Handlers
// ===================================

pub async fn list_keys_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ApiKey>>, (StatusCode, String)> {
    let keys = list_api_keys(&state.db).await.map_err(|e| {
        error!("List keys failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(keys))
}

pub async fn create_key_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateKeyRequest>,
) -> Result<Json<ApiKeyResponse>, (StatusCode, String)> {
    let (raw_key, key_hash, prefix) = KeyManager::generate_api_key();

    let key = ApiKey {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        key_hash,
        prefix,
        owner_id: req.owner_id,
        scopes: req.scopes,
        expires_at: None, // Optional expiration
        created_at: chrono::Utc::now(),
        last_used_at: None,
    };

    let created = create_api_key(&state.db, key).await.map_err(|e| {
        error!("Create key failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(ApiKeyResponse {
        key: created,
        raw_key,
    }))
}

pub async fn delete_key_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    delete_api_key(&state.db, &id).await.map_err(|e| {
        error!("Delete key failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(StatusCode::NO_CONTENT)
}

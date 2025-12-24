use axum::{
    extract::{State, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::state::AppState;
use gateway_core::auth::User;
use surreal_config::auth_schema::get_user_by_username;
use naseej_security::{KeyManager, JwtIssuer, AuthConfig};

// ===================================
// Types
// ===================================

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
    pub expires_at: u64,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub roles: Vec<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            roles: user.roles,
        }
    }
}

// ===================================
// Handlers
// ===================================

/// Login endpoint - /api/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    // Fetch user
    let user = get_user_by_username(&state.db, &request.username)
        .await
        .map_err(|e| {
            error!("Login DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Verify password
    if !KeyManager::verify_password(&request.password, &user.password_hash) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    if !user.active {
        return Err((StatusCode::FORBIDDEN, "Account disabled".to_string()));
    }

    // Issue Token
    // TODO: Load config from state/env properly. For now valid default.
    let config = AuthConfig::default(); 
    let issuer = JwtIssuer::new(config).map_err(|e| {
        error!("JWT Issuer init failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Auth error".to_string())
    })?;

    let (token, expires_at) = issuer.issue_token(&user.id, user.roles.clone())
        .map_err(|e| {
            error!("Token issuance failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Token error".to_string())
        })?;

    info!(user = %user.username, "User logged in successfully");

    Ok(Json(LoginResponse {
        token,
        user: user.into(),
        expires_at,
    }))
}

//! Internal API handlers for service-to-service communication
//! These endpoints are NOT meant for external access and should be protected in production.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::handlers::AppState;
use crate::models::Role;
use crate::user_service::{RefreshTokenService, UserService};

// ============================================================================
// User Internal API
// ============================================================================

/// GET /internal/users/count - Get total user count
pub async fn get_user_count(State(state): State<AppState>) -> impl IntoResponse {
    let user_service = UserService::new(state.pool.clone());
    
    match user_service.count().await {
        Ok(count) => (StatusCode::OK, Json(UserCountResponse { count })).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(InternalError { error: err.to_string() }),
        )
            .into_response(),
    }
}

#[derive(Serialize)]
pub struct UserCountResponse {
    pub count: i64,
}

/// GET /internal/users/by-email/{email} - Find user by email (includes password hash)
pub async fn get_user_by_email(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> impl IntoResponse {
    let user_service = UserService::new(state.pool.clone());
    
    match user_service.find_by_email(&email).await {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(InternalError { error: err.to_string() }),
        )
            .into_response(),
    }
}

/// GET /internal/users/{id} - Find user by ID (includes password hash)
pub async fn get_user_by_id_internal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user_service = UserService::new(state.pool.clone());
    
    match user_service.find_by_id(id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(InternalError { error: err.to_string() }),
        )
            .into_response(),
    }
}

/// POST /internal/users - Create user with password hash
#[derive(Deserialize)]
pub struct CreateUserInternalRequest {
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub organisation_id: Option<Uuid>,
    pub role: Option<Role>,
}

pub async fn create_user_internal(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserInternalRequest>,
) -> impl IntoResponse {
    let user_service = UserService::new(state.pool.clone());
    let role = payload.role.unwrap_or(Role::User);
    
    match user_service.create(
        &payload.email,
        &payload.name,
        &payload.password_hash,
        payload.organisation_id,
        role,
    ).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(InternalError { error: err.to_string() }),
        )
            .into_response(),
    }
}

// ============================================================================
// Refresh Token Internal API
// ============================================================================

/// POST /internal/refresh-tokens - Create a new refresh token
#[derive(Deserialize)]
pub struct CreateRefreshTokenRequest {
    pub user_id: Uuid,
    pub organisation_id: Option<Uuid>,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct CreateRefreshTokenResponse {
    pub id: Uuid,
}

pub async fn create_refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<CreateRefreshTokenRequest>,
) -> impl IntoResponse {
    let token_service = RefreshTokenService::new(state.pool.clone());
    
    match token_service.create(
        payload.user_id,
        payload.organisation_id,
        &payload.token_hash,
        payload.expires_at,
    ).await {
        Ok(id) => (StatusCode::CREATED, Json(CreateRefreshTokenResponse { id })).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(InternalError { error: err.to_string() }),
        )
            .into_response(),
    }
}

/// GET /internal/refresh-tokens/by-hash/{hash} - Find refresh token by hash
#[derive(Serialize)]
pub struct RefreshTokenInfoResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organisation_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
}

pub async fn get_refresh_token_by_hash(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    let token_service = RefreshTokenService::new(state.pool.clone());
    
    match token_service.find_by_hash(&hash).await {
        Ok(Some(info)) => (
            StatusCode::OK,
            Json(RefreshTokenInfoResponse {
                id: info.id,
                user_id: info.user_id,
                organisation_id: info.organisation_id,
                expires_at: info.expires_at,
            }),
        )
            .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(InternalError { error: err.to_string() }),
        )
            .into_response(),
    }
}

/// PUT /internal/refresh-tokens/{id} - Update (rotate) refresh token
#[derive(Deserialize)]
pub struct UpdateRefreshTokenRequest {
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn update_refresh_token(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRefreshTokenRequest>,
) -> impl IntoResponse {
    let token_service = RefreshTokenService::new(state.pool.clone());
    
    match token_service.update(id, &payload.token_hash, payload.expires_at).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(InternalError { error: err.to_string() }),
        )
            .into_response(),
    }
}

/// DELETE /internal/refresh-tokens/{id} - Delete refresh token by ID
pub async fn delete_refresh_token(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let token_service = RefreshTokenService::new(state.pool.clone());
    
    match token_service.delete(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// DELETE /internal/refresh-tokens/by-hash/{hash} - Delete refresh token by hash
pub async fn delete_refresh_token_by_hash(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    let token_service = RefreshTokenService::new(state.pool.clone());
    
    match token_service.delete_by_hash(&hash).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Serialize)]
struct InternalError {
    error: String,
}

use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use uuid::Uuid;

use contracts::{RefreshTokenServiceContract, Role, UserServiceContract, UserWithPassword};

use crate::config::AuthConfig;
use crate::models::{
    AuthResponse, AuthUserInfo, ErrorResponse, LoginRequest, RefreshRequest, ValidateResponse,
};
use crate::token::{
    generate_access_token, generate_refresh_token, hash_password, hash_refresh_token,
    validate_access_token, verify_password,
};

/// Shared application state using trait objects for flexibility
#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<dyn UserServiceContract>,
    pub token_service: Arc<dyn RefreshTokenServiceContract>,
    pub config: Arc<AuthConfig>,
}

/// POST /auth/login - Authenticate user with email and password
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Check if there are any users in the database via contract
    let user_count = state.user_service.count().await.unwrap_or(0);

    let user: Option<UserWithPassword> = if user_count == 0 {
        // No users exist - check default admin credentials
        if let (Some(default_email), Some(default_password)) = (
            &state.config.default_admin_email,
            &state.config.default_admin_password,
        ) {
            if payload.email == *default_email && payload.password == *default_password {
                // Create a virtual user for the default admin (not stored in DB)
                Some(UserWithPassword {
                    id: Uuid::nil(), // Special ID for default admin
                    organisation_id: None,
                    email: default_email.clone(),
                    name: "Default Admin".to_string(),
                    password_hash: None,
                    role: Role::SuperAdmin,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        // Users exist - authenticate against database via contract
        // Default admin is disabled when real users exist
        let db_user = state
            .user_service
            .find_by_email(&payload.email)
            .await
            .unwrap_or(None);

        if let Some(ref user) = db_user {
            // Verify password
            if let Some(ref hash) = user.password_hash {
                match verify_password(&payload.password, hash) {
                    Ok(true) => db_user,
                    _ => None,
                }
            } else {
                // User has no password set
                None
            }
        } else {
            None
        }
    };

    let Some(user) = user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
                message: "Invalid email or password".to_string(),
            }),
        )
            .into_response();
    };

    // Generate tokens
    let access_token = match generate_access_token(&user, &state.config) {
        Ok(token) => token,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "token_error".to_string(),
                    message: format!("Failed to generate token: {}", err),
                }),
            )
                .into_response();
        }
    };

    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    // Store refresh token via contract (only for real users, not default admin)
    if user.id != Uuid::nil() {
        let expires_at = Utc::now() + Duration::days(7); // Refresh token valid for 7 days
        let _ = state
            .token_service
            .create(user.id, user.organisation_id, &refresh_token_hash, expires_at)
            .await;
    }

    (
        StatusCode::OK,
        Json(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: state.config.token_ttl_seconds,
            user: AuthUserInfo::from(&user),
        }),
    )
        .into_response()
}

/// POST /auth/refresh - Refresh access token using refresh token
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    let token_hash = hash_refresh_token(&payload.refresh_token);

    // Find the refresh token via contract
    let result = state
        .token_service
        .find_by_hash(&token_hash)
        .await
        .unwrap_or(None);

    let Some(token_info) = result else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "invalid_token".to_string(),
                message: "Invalid or expired refresh token".to_string(),
            }),
        )
            .into_response();
    };

    // Check if token is expired
    if token_info.expires_at < Utc::now() {
        // Delete expired token
        let _ = state.token_service.delete(token_info.id).await;

        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "expired_token".to_string(),
                message: "Refresh token has expired".to_string(),
            }),
        )
            .into_response();
    }

    // Get user via contract
    let user = state
        .user_service
        .find_by_id(token_info.user_id)
        .await
        .unwrap_or(None);

    let Some(user) = user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
                message: "User not found".to_string(),
            }),
        )
            .into_response();
    };

    // Generate new access token
    let access_token = match generate_access_token(&user, &state.config) {
        Ok(token) => token,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "token_error".to_string(),
                    message: format!("Failed to generate token: {}", err),
                }),
            )
                .into_response();
        }
    };

    // Generate new refresh token (rotate)
    let new_refresh_token = generate_refresh_token();
    let new_refresh_token_hash = hash_refresh_token(&new_refresh_token);
    let new_expires_at = Utc::now() + Duration::days(7);

    // Update refresh token via contract
    let _ = state
        .token_service
        .update(token_info.id, &new_refresh_token_hash, new_expires_at)
        .await;

    (
        StatusCode::OK,
        Json(AuthResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: state.config.token_ttl_seconds,
            user: AuthUserInfo::from(&user),
        }),
    )
        .into_response()
}

/// GET /auth/validate - Validate an access token (for internal service use)
pub async fn validate(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return (
            StatusCode::OK,
            Json(ValidateResponse {
                valid: false,
                user_id: None,
                email: None,
                expires_at: None,
            }),
        );
    };

    match validate_access_token(token, &state.config) {
        Ok(claims) => (
            StatusCode::OK,
            Json(ValidateResponse {
                valid: true,
                user_id: Some(claims.sub),
                email: Some(claims.email),
                expires_at: Some(claims.exp),
            }),
        ),
        Err(_) => (
            StatusCode::OK,
            Json(ValidateResponse {
                valid: false,
                user_id: None,
                email: None,
                expires_at: None,
            }),
        ),
    }
}

/// POST /auth/logout - Invalidate refresh token
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    let token_hash = hash_refresh_token(&payload.refresh_token);
    let _ = state.token_service.delete_by_hash(&token_hash).await;
    StatusCode::NO_CONTENT
}

/// POST /auth/register - Register a new user with password
#[derive(serde::Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Hash password
    let password_hash = match hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "hash_error".to_string(),
                    message: format!("Failed to hash password: {}", err),
                }),
            )
                .into_response();
        }
    };

    // Create user via contract (default role is User, no organisation)
    let result = state
        .user_service
        .create(&payload.email, &payload.name, &password_hash, None, Role::User)
        .await;

    match result {
        Ok(user) => {
            // Generate tokens
            let access_token = match generate_access_token(&user, &state.config) {
                Ok(token) => token,
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "token_error".to_string(),
                            message: format!("Failed to generate token: {}", err),
                        }),
                    )
                        .into_response();
                }
            };

            let refresh_token = generate_refresh_token();
            let refresh_token_hash = hash_refresh_token(&refresh_token);
            let expires_at = Utc::now() + Duration::days(7);

            let _ = state
                .token_service
                .create(user.id, user.organisation_id, &refresh_token_hash, expires_at)
                .await;

            (
                StatusCode::CREATED,
                Json(AuthResponse {
                    access_token,
                    refresh_token,
                    token_type: "Bearer".to_string(),
                    expires_in: state.config.token_ttl_seconds,
                    user: AuthUserInfo::from(&user),
                }),
            )
                .into_response()
        }
        Err(err) => {
            let message = match err {
                contracts::ContractError::AlreadyExists => "Email already registered".to_string(),
                _ => format!("Failed to create user: {}", err),
            };

            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "registration_failed".to_string(),
                    message,
                }),
            )
                .into_response()
        }
    }
}

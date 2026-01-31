use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export user types from admin_core
pub use admin_core::{UserInfo, UserWithPassword};

/// Login request payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Token refresh request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Authentication response with tokens
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: AuthUserInfo,
}

/// User info for auth responses
#[derive(Debug, Serialize)]
pub struct AuthUserInfo {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

impl From<&UserWithPassword> for AuthUserInfo {
    fn from(user: &UserWithPassword) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
        }
    }
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Expiration time (Unix timestamp)
    pub exp: u64,
    /// Issued at (Unix timestamp)
    pub iat: u64,
    /// User email
    pub email: String,
    /// User name
    pub name: String,
}

/// Token validation response
#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub expires_at: Option<u64>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

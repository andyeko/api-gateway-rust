//! Shared types used across service contracts

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Error type for contract operations
#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("Not found")]
    NotFound,

    #[error("Already exists")]
    AlreadyExists,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

pub type ContractResult<T> = Result<T, ContractError>;

/// User roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    SuperAdmin,
    Admin,
    Supervisor,
    #[default]
    User,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::SuperAdmin => "SUPER_ADMIN",
            Role::Admin => "ADMIN",
            Role::Supervisor => "SUPERVISOR",
            Role::User => "USER",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "SUPER_ADMIN" => Role::SuperAdmin,
            "ADMIN" => Role::Admin,
            "SUPERVISOR" => Role::Supervisor,
            _ => Role::User,
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Organisation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganisationInfo {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User data with password hash for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithPassword {
    pub id: Uuid,
    pub organisation_id: Option<Uuid>,
    pub email: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public user info (no password hash)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub organisation_id: Option<Uuid>,
    pub email: String,
    pub name: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&UserWithPassword> for UserInfo {
    fn from(user: &UserWithPassword) -> Self {
        Self {
            id: user.id,
            organisation_id: user.organisation_id,
            email: user.email.clone(),
            name: user.name.clone(),
            role: user.role,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Refresh token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organisation_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
}

//! Refresh token service contract

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::types::{ContractResult, RefreshTokenInfo};

/// Contract for refresh token operations
/// 
/// Implementations:
/// - `InMemoryRefreshTokenService` - Direct database access (for monolith mode)
/// - `HttpRefreshTokenService` - HTTP calls to admin service (for microservice mode)
#[async_trait]
pub trait RefreshTokenServiceContract: Send + Sync {
    /// Store a new refresh token
    async fn create(
        &self,
        user_id: Uuid,
        organisation_id: Option<Uuid>,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> ContractResult<Uuid>;

    /// Find refresh token by hash
    async fn find_by_hash(&self, token_hash: &str) -> ContractResult<Option<RefreshTokenInfo>>;

    /// Update refresh token (for rotation)
    async fn update(
        &self,
        token_id: Uuid,
        new_token_hash: &str,
        new_expires_at: DateTime<Utc>,
    ) -> ContractResult<()>;

    /// Delete refresh token by hash
    async fn delete_by_hash(&self, token_hash: &str) -> ContractResult<()>;

    /// Delete refresh token by ID
    async fn delete(&self, token_id: Uuid) -> ContractResult<()>;
}

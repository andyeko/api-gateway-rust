//! User service contract

use async_trait::async_trait;
use uuid::Uuid;

use crate::types::{ContractResult, Role, UserWithPassword};

/// Contract for user-related operations
/// 
/// Implementations:
/// - `InMemoryUserService` - Direct database access (for monolith mode)
/// - `HttpUserService` - HTTP calls to admin service (for microservice mode)
#[async_trait]
pub trait UserServiceContract: Send + Sync {
    /// Get total count of users in database
    async fn count(&self) -> ContractResult<i64>;

    /// Find user by email with password hash for authentication
    async fn find_by_email(&self, email: &str) -> ContractResult<Option<UserWithPassword>>;

    /// Find user by ID with password hash
    async fn find_by_id(&self, id: Uuid) -> ContractResult<Option<UserWithPassword>>;

    /// Create a new user with password hash
    async fn create(
        &self,
        email: &str,
        name: &str,
        password_hash: &str,
        organisation_id: Option<Uuid>,
        role: Role,
    ) -> ContractResult<UserWithPassword>;

    /// Update user's password hash
    async fn update_password(&self, user_id: Uuid, password_hash: &str) -> ContractResult<()>;
}

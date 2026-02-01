//! In-memory implementations of contracts for monolith mode
//! 
//! These implementations directly access the database without network overhead.
//! Used when running all services as a single binary.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use contracts::{
    ContractError, ContractResult, RefreshTokenInfo, RefreshTokenServiceContract, Role,
    UserServiceContract, UserWithPassword,
};

use crate::db::DbPool;

// ============================================================================
// In-Memory User Service Implementation
// ============================================================================

/// Direct database implementation of UserServiceContract
/// Used in monolith mode - no network overhead
#[derive(Clone)]
pub struct InMemoryUserService {
    pool: DbPool,
}

impl InMemoryUserService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

/// Internal struct for database queries
#[derive(sqlx::FromRow)]
struct DbUserWithPassword {
    id: Uuid,
    organisation_id: Option<Uuid>,
    email: String,
    name: String,
    password_hash: Option<String>,
    role: crate::models::Role,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<DbUserWithPassword> for UserWithPassword {
    fn from(u: DbUserWithPassword) -> Self {
        Self {
            id: u.id,
            organisation_id: u.organisation_id,
            email: u.email,
            name: u.name,
            password_hash: u.password_hash,
            role: match u.role {
                crate::models::Role::SuperAdmin => Role::SuperAdmin,
                crate::models::Role::Admin => Role::Admin,
                crate::models::Role::Supervisor => Role::Supervisor,
                crate::models::Role::User => Role::User,
            },
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

fn contract_role_to_db(role: Role) -> crate::models::Role {
    match role {
        Role::SuperAdmin => crate::models::Role::SuperAdmin,
        Role::Admin => crate::models::Role::Admin,
        Role::Supervisor => crate::models::Role::Supervisor,
        Role::User => crate::models::Role::User,
    }
}

#[async_trait]
impl UserServiceContract for InMemoryUserService {
    async fn count(&self) -> ContractResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(count)
    }

    async fn find_by_email(&self, email: &str) -> ContractResult<Option<UserWithPassword>> {
        let user = sqlx::query_as::<_, DbUserWithPassword>(
            "SELECT id, organisation_id, email, name, password_hash, role, created_at, updated_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(user.map(Into::into))
    }

    async fn find_by_id(&self, id: Uuid) -> ContractResult<Option<UserWithPassword>> {
        let user = sqlx::query_as::<_, DbUserWithPassword>(
            "SELECT id, organisation_id, email, name, password_hash, role, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(user.map(Into::into))
    }

    async fn create(
        &self,
        email: &str,
        name: &str,
        password_hash: &str,
        organisation_id: Option<Uuid>,
        role: Role,
    ) -> ContractResult<UserWithPassword> {
        let id = Uuid::new_v4();
        let db_role = contract_role_to_db(role);
        let user = sqlx::query_as::<_, DbUserWithPassword>(
            r#"
            INSERT INTO users (id, organisation_id, email, name, password_hash, role)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, organisation_id, email, name, password_hash, role, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(organisation_id)
        .bind(email)
        .bind(name)
        .bind(password_hash)
        .bind(db_role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique") {
                ContractError::AlreadyExists
            } else {
                ContractError::Internal(e.to_string())
            }
        })?;
        Ok(user.into())
    }

    async fn update_password(&self, user_id: Uuid, password_hash: &str) -> ContractResult<()> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(())
    }
}

// ============================================================================
// In-Memory Refresh Token Service Implementation
// ============================================================================

/// Direct database implementation of RefreshTokenServiceContract
/// Used in monolith mode - no network overhead
#[derive(Clone)]
pub struct InMemoryRefreshTokenService {
    pool: DbPool,
}

impl InMemoryRefreshTokenService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

/// Internal struct for database queries
#[derive(sqlx::FromRow)]
struct DbRefreshTokenInfo {
    id: Uuid,
    user_id: Uuid,
    organisation_id: Option<Uuid>,
    expires_at: DateTime<Utc>,
}

impl From<DbRefreshTokenInfo> for RefreshTokenInfo {
    fn from(t: DbRefreshTokenInfo) -> Self {
        Self {
            id: t.id,
            user_id: t.user_id,
            organisation_id: t.organisation_id,
            expires_at: t.expires_at,
        }
    }
}

#[async_trait]
impl RefreshTokenServiceContract for InMemoryRefreshTokenService {
    async fn create(
        &self,
        user_id: Uuid,
        organisation_id: Option<Uuid>,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> ContractResult<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, organisation_id, token_hash, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(organisation_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(id)
    }

    async fn find_by_hash(&self, token_hash: &str) -> ContractResult<Option<RefreshTokenInfo>> {
        let result = sqlx::query_as::<_, DbRefreshTokenInfo>(
            r#"
            SELECT id, user_id, organisation_id, expires_at
            FROM refresh_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ContractError::Internal(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn update(
        &self,
        token_id: Uuid,
        new_token_hash: &str,
        new_expires_at: DateTime<Utc>,
    ) -> ContractResult<()> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET token_hash = $1, expires_at = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(new_token_hash)
        .bind(new_expires_at)
        .bind(token_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn delete_by_hash(&self, token_hash: &str) -> ContractResult<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, token_id: Uuid) -> ContractResult<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
            .bind(token_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(())
    }
}

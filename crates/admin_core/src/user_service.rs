use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::DbPool;

/// User data with password hash for authentication
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserWithPassword {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Public user info (no password hash)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl From<&UserWithPassword> for UserInfo {
    fn from(user: &UserWithPassword) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
            created_at: user.created_at,
        }
    }
}

/// User service for operations on users
#[derive(Clone)]
pub struct UserService {
    pool: DbPool,
}

impl UserService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get total count of users in database
    pub async fn count(&self) -> anyhow::Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    /// Find user by email with password hash for authentication
    pub async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<UserWithPassword>> {
        let user = sqlx::query_as::<_, UserWithPassword>(
            "SELECT id, email, name, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    /// Find user by ID with password hash
    pub async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<UserWithPassword>> {
        let user = sqlx::query_as::<_, UserWithPassword>(
            "SELECT id, email, name, password_hash, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    /// Create a new user with password hash
    pub async fn create(
        &self,
        email: &str,
        name: &str,
        password_hash: &str,
    ) -> anyhow::Result<UserWithPassword> {
        let id = Uuid::new_v4();
        let user = sqlx::query_as::<_, UserWithPassword>(
            r#"
            INSERT INTO users (id, email, name, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, name, password_hash, created_at
            "#,
        )
        .bind(id)
        .bind(email)
        .bind(name)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    /// Update user's password hash
    pub async fn update_password(&self, user_id: Uuid, password_hash: &str) -> anyhow::Result<()> {
        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// Refresh token service
#[derive(Clone)]
pub struct RefreshTokenService {
    pool: DbPool,
}

impl RefreshTokenService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Store a new refresh token
    pub async fn create(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    /// Find refresh token by hash
    pub async fn find_by_hash(
        &self,
        token_hash: &str,
    ) -> anyhow::Result<Option<(Uuid, Uuid, DateTime<Utc>)>> {
        let result: Option<(Uuid, Uuid, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT id, user_id, expires_at
            FROM refresh_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    /// Update refresh token (for rotation)
    pub async fn update(
        &self,
        token_id: Uuid,
        new_token_hash: &str,
        new_expires_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET token_hash = $1, expires_at = $2, created_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(new_token_hash)
        .bind(new_expires_at)
        .bind(token_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete refresh token by hash
    pub async fn delete_by_hash(&self, token_hash: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete refresh token by ID
    pub async fn delete(&self, token_id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
            .bind(token_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

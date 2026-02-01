//! HTTP implementations of contracts for microservice mode
//! 
//! These implementations call the admin service via HTTP.
//! Used when running services as separate processes.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use contracts::{
    ContractError, ContractResult, RefreshTokenInfo, RefreshTokenServiceContract, Role,
    UserServiceContract, UserWithPassword,
};

// ============================================================================
// HTTP User Service Implementation
// ============================================================================

/// HTTP implementation of UserServiceContract
/// Used in microservice mode - calls admin service via network
#[derive(Clone)]
pub struct HttpUserService {
    base_url: String,
    client: reqwest::Client,
}

impl HttpUserService {
    pub fn new(admin_base_url: &str) -> Self {
        Self {
            base_url: admin_base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl UserServiceContract for HttpUserService {
    async fn count(&self) -> ContractResult<i64> {
        let url = format!("{}/internal/users/count", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ContractError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ContractError::Internal(format!(
                "Failed to get user count: {}",
                resp.status()
            )));
        }

        #[derive(Deserialize)]
        struct CountResponse {
            count: i64,
        }

        let data: CountResponse = resp
            .json()
            .await
            .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(data.count)
    }

    async fn find_by_email(&self, email: &str) -> ContractResult<Option<UserWithPassword>> {
        let url = format!(
            "{}/internal/users/by-email/{}",
            self.base_url,
            urlencoding::encode(email)
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ContractError::Connection(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !resp.status().is_success() {
            return Err(ContractError::Internal(format!(
                "Failed to find user by email: {}",
                resp.status()
            )));
        }

        let user: UserWithPassword = resp
            .json()
            .await
            .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(Some(user))
    }

    async fn find_by_id(&self, id: Uuid) -> ContractResult<Option<UserWithPassword>> {
        let url = format!("{}/internal/users/{}", self.base_url, id);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ContractError::Connection(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !resp.status().is_success() {
            return Err(ContractError::Internal(format!(
                "Failed to find user by ID: {}",
                resp.status()
            )));
        }

        let user: UserWithPassword = resp
            .json()
            .await
            .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(Some(user))
    }

    async fn create(
        &self,
        email: &str,
        name: &str,
        password_hash: &str,
        organisation_id: Option<Uuid>,
        role: Role,
    ) -> ContractResult<UserWithPassword> {
        let url = format!("{}/internal/users", self.base_url);

        #[derive(Serialize)]
        struct CreateUserRequest<'a> {
            email: &'a str,
            name: &'a str,
            password_hash: &'a str,
            organisation_id: Option<Uuid>,
            role: Role,
        }

        let resp = self
            .client
            .post(&url)
            .json(&CreateUserRequest {
                email,
                name,
                password_hash,
                organisation_id,
                role,
            })
            .send()
            .await
            .map_err(|e| ContractError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            if status == reqwest::StatusCode::CONFLICT {
                return Err(ContractError::AlreadyExists);
            }
            return Err(ContractError::Internal(format!(
                "Failed to create user: {}",
                status
            )));
        }

        let user: UserWithPassword = resp
            .json()
            .await
            .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(user)
    }

    async fn update_password(&self, user_id: Uuid, password_hash: &str) -> ContractResult<()> {
        let url = format!("{}/internal/users/{}/password", self.base_url, user_id);

        #[derive(Serialize)]
        struct UpdatePasswordRequest<'a> {
            password_hash: &'a str,
        }

        let resp = self
            .client
            .put(&url)
            .json(&UpdatePasswordRequest { password_hash })
            .send()
            .await
            .map_err(|e| ContractError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ContractError::Internal(format!(
                "Failed to update password: {}",
                resp.status()
            )));
        }

        Ok(())
    }
}

// ============================================================================
// HTTP Refresh Token Service Implementation
// ============================================================================

/// HTTP implementation of RefreshTokenServiceContract
/// Used in microservice mode - calls admin service via network
#[derive(Clone)]
pub struct HttpRefreshTokenService {
    base_url: String,
    client: reqwest::Client,
}

impl HttpRefreshTokenService {
    pub fn new(admin_base_url: &str) -> Self {
        Self {
            base_url: admin_base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl RefreshTokenServiceContract for HttpRefreshTokenService {
    async fn create(
        &self,
        user_id: Uuid,
        organisation_id: Option<Uuid>,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> ContractResult<Uuid> {
        let url = format!("{}/internal/refresh-tokens", self.base_url);

        #[derive(Serialize)]
        struct CreateTokenRequest<'a> {
            user_id: Uuid,
            organisation_id: Option<Uuid>,
            token_hash: &'a str,
            expires_at: DateTime<Utc>,
        }

        let resp = self
            .client
            .post(&url)
            .json(&CreateTokenRequest {
                user_id,
                organisation_id,
                token_hash,
                expires_at,
            })
            .send()
            .await
            .map_err(|e| ContractError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ContractError::Internal(format!(
                "Failed to create refresh token: {}",
                resp.status()
            )));
        }

        #[derive(Deserialize)]
        struct CreateTokenResponse {
            id: Uuid,
        }

        let data: CreateTokenResponse = resp
            .json()
            .await
            .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(data.id)
    }

    async fn find_by_hash(&self, token_hash: &str) -> ContractResult<Option<RefreshTokenInfo>> {
        let url = format!(
            "{}/internal/refresh-tokens/by-hash/{}",
            self.base_url,
            urlencoding::encode(token_hash)
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ContractError::Connection(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !resp.status().is_success() {
            return Err(ContractError::Internal(format!(
                "Failed to find refresh token: {}",
                resp.status()
            )));
        }

        let info: RefreshTokenInfo = resp
            .json()
            .await
            .map_err(|e| ContractError::Internal(e.to_string()))?;
        Ok(Some(info))
    }

    async fn update(
        &self,
        token_id: Uuid,
        new_token_hash: &str,
        new_expires_at: DateTime<Utc>,
    ) -> ContractResult<()> {
        let url = format!("{}/internal/refresh-tokens/{}", self.base_url, token_id);

        #[derive(Serialize)]
        struct UpdateTokenRequest<'a> {
            token_hash: &'a str,
            expires_at: DateTime<Utc>,
        }

        let resp = self
            .client
            .put(&url)
            .json(&UpdateTokenRequest {
                token_hash: new_token_hash,
                expires_at: new_expires_at,
            })
            .send()
            .await
            .map_err(|e| ContractError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ContractError::Internal(format!(
                "Failed to update refresh token: {}",
                resp.status()
            )));
        }

        Ok(())
    }

    async fn delete_by_hash(&self, token_hash: &str) -> ContractResult<()> {
        let url = format!(
            "{}/internal/refresh-tokens/by-hash/{}",
            self.base_url,
            urlencoding::encode(token_hash)
        );
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| ContractError::Connection(e.to_string()))?;

        if !resp.status().is_success() && resp.status() != reqwest::StatusCode::NOT_FOUND {
            return Err(ContractError::Internal(format!(
                "Failed to delete refresh token: {}",
                resp.status()
            )));
        }

        Ok(())
    }

    async fn delete(&self, token_id: Uuid) -> ContractResult<()> {
        let url = format!("{}/internal/refresh-tokens/{}", self.base_url, token_id);
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| ContractError::Connection(e.to_string()))?;

        if !resp.status().is_success() && resp.status() != reqwest::StatusCode::NOT_FOUND {
            return Err(ContractError::Internal(format!(
                "Failed to delete refresh token: {}",
                resp.status()
            )));
        }

        Ok(())
    }
}

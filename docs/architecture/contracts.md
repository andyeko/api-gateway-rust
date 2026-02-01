# Service Contracts

The **contracts crate** is the foundation of the modular monolith architecture. It defines service interfaces as Rust traits, enabling the same business logic to run with different backends.

---

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    contracts crate                          │
│  ┌─────────────────────┐  ┌─────────────────────────────┐   │
│  │ UserServiceContract │  │ RefreshTokenServiceContract │   │
│  └──────────┬──────────┘  └──────────────┬──────────────┘   │
│             │                            │                  │
└─────────────│────────────────────────────│──────────────────┘
              │                            │
    ┌─────────┴─────────┐        ┌─────────┴─────────┐
    │                   │        │                   │
┌───▼───┐          ┌────▼────┐  ┌───▼───┐       ┌────▼────┐
│InMemory│          │  HTTP   │  │InMemory│       │  HTTP   │
│  User  │          │  User   │  │ Token  │       │ Token   │
│Service │          │ Service │  │Service │       │Service  │
└───┬────┘          └────┬────┘  └───┬────┘       └────┬────┘
    │                    │           │                 │
    ▼                    ▼           ▼                 ▼
 Database            HTTP API     Database          HTTP API
```

---

## Traits

### UserServiceContract

Defines operations for user management:

```rust
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
    ) -> ContractResult<UserWithPassword>;

    /// Update user's password hash
    async fn update_password(&self, user_id: Uuid, password_hash: &str) -> ContractResult<()>;
}
```

### RefreshTokenServiceContract

Defines operations for refresh token management:

```rust
#[async_trait]
pub trait RefreshTokenServiceContract: Send + Sync {
    /// Create a new refresh token
    async fn create(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> ContractResult<RefreshTokenInfo>;

    /// Find refresh token by its hash
    async fn find_by_hash(&self, token_hash: &str) -> ContractResult<Option<RefreshTokenInfo>>;

    /// Update refresh token (new hash and expiry)
    async fn update(
        &self,
        id: Uuid,
        new_token_hash: &str,
        new_expires_at: DateTime<Utc>,
    ) -> ContractResult<()>;

    /// Delete refresh token by hash (logout)
    async fn delete_by_hash(&self, token_hash: &str) -> ContractResult<()>;

    /// Delete refresh token by ID
    async fn delete(&self, id: Uuid) -> ContractResult<()>;
}
```

---

## Shared Types

### ContractError

Standard error type for all contract operations:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("not found")]
    NotFound,
    
    #[error("already exists")]
    AlreadyExists,
    
    #[error("internal error: {0}")]
    Internal(String),
    
    #[error("connection error: {0}")]
    Connection(String),
}
```

### Data Types

```rust
/// User with password hash (for authentication)
pub struct UserWithPassword {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Refresh token information
pub struct RefreshTokenInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
```

---

## Implementations

### In-Memory Implementations (Monolith)

Located in `admin_core/src/contract_impl.rs`:

```rust
pub struct InMemoryUserService {
    pool: DbPool,
}

pub struct InMemoryRefreshTokenService {
    pool: DbPool,
}
```

- Direct database access via sqlx
- No serialization overhead
- Shared database pool with other services

### HTTP Implementations (Microservices)

Located in `auth_core/src/http_client.rs`:

```rust
pub struct HttpUserService {
    base_url: String,
    client: reqwest::Client,
}

pub struct HttpRefreshTokenService {
    base_url: String,
    client: reqwest::Client,
}
```

- Calls admin service `/internal/*` endpoints
- JSON serialization/deserialization
- Network latency (but enables independent scaling)

---

## Usage

### Monolith Mode (app/modules/gateway.rs)

```rust
use admin_core::{InMemoryUserService, InMemoryRefreshTokenService};

// Create in-memory implementations with shared pool
let user_service = Arc::new(InMemoryUserService::new(pool.clone()));
let token_service = Arc::new(InMemoryRefreshTokenService::new(pool.clone()));

// Build auth router with in-memory services
let router = auth_core::server::build_inner_router(
    user_service,
    token_service,
    Arc::new(config),
);
```

### Microservices Mode (auth_core/src/service.rs)

```rust
use crate::http_client::{HttpUserService, HttpRefreshTokenService};

// Create HTTP implementations
let user_service = Arc::new(HttpUserService::new(&config.admin_service_url));
let token_service = Arc::new(HttpRefreshTokenService::new(&config.admin_service_url));

// Run auth server with HTTP services
auth_core::server::run(&config.listen_addr, user_service, token_service, config).await
```

---

## Benefits

1. **Compile-time abstraction**: Traits are resolved at compile time
2. **Zero runtime overhead in monolith**: No dynamic dispatch needed
3. **Clear API boundaries**: Contracts enforce stable interfaces
4. **Easy testing**: Create mock implementations for unit tests
5. **Flexible deployment**: Same code, different wiring

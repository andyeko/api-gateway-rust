# Admin Service

## Purpose
User management, database operations, and internal APIs for service-to-service communication.

## Ownership
- Core crate: `crates/admin_core`
- Microservice: `services/admin`
- Module (monolith): `app/modules/admin`
- Contract implementations: `InMemoryUserService`, `InMemoryRefreshTokenService`

## Responsibilities
- Users CRUD API (public)
- Internal APIs for auth service (service-to-service)
- Database migrations and connection pooling
- Refresh token storage

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgres://postgres:postgres@localhost:5432/apisentinel` | PostgreSQL connection |
| `ADMIN_BIND_ADDR` | `0.0.0.0:4001` | Listen address |

## Public API (Users CRUD)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/users` | GET | List users (supports `_start`, `_end` for react-admin) |
| `/users/:id` | GET | Get user by ID |
| `/users` | POST | Create user |
| `/users/:id` | PUT | Update user |
| `/users/:id` | DELETE | Delete user |

## Internal API (Service-to-Service)

Used by auth service in microservices mode via `HttpUserService` and `HttpRefreshTokenService`.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/internal/users/count` | GET | Get total user count |
| `/internal/users/by-email/{email}` | GET | Find user by email (with password hash) |
| `/internal/users/{id}` | GET | Get user by ID (with password hash) |
| `/internal/users` | POST | Create user with password hash |
| `/internal/refresh-tokens` | POST | Create refresh token |
| `/internal/refresh-tokens/by-hash/{hash}` | GET | Find refresh token by hash |
| `/internal/refresh-tokens/by-hash/{hash}` | DELETE | Delete refresh token by hash |
| `/internal/refresh-tokens/{id}` | PUT | Update refresh token |
| `/internal/refresh-tokens/{id}` | DELETE | Delete refresh token |

## Contract Implementations

In monolith mode, auth uses these implementations directly (no HTTP):

```rust
// In-memory implementations (direct database access)
pub struct InMemoryUserService { pool: DbPool }
pub struct InMemoryRefreshTokenService { pool: DbPool }
```

## Database Schema

### users
| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| email | VARCHAR | Unique email |
| name | VARCHAR | Display name |
| password_hash | VARCHAR | Argon2 hash |
| created_at | TIMESTAMP | Creation time |
| updated_at | TIMESTAMP | Last update |

### refresh_tokens
| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Foreign key to users |
| token_hash | VARCHAR | SHA-256 hash of token |
| expires_at | TIMESTAMP | Expiration time |
| created_at | TIMESTAMP | Creation time |

## Notes
- Runs on port 4001 in microservices mode
- Embedded in gateway on port 4000 in monolith mode
- Internal APIs are not exposed through gateway proxy

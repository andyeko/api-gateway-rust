# Auth Service

## Purpose
Authentication and authorization using JWT tokens with refresh token rotation.

## Ownership
- Core crate: `crates/auth_core`
- Microservice: `services/auth`
- Module (monolith): `app/modules/auth`
- HTTP implementations: `HttpUserService`, `HttpRefreshTokenService`

## Responsibilities
- User login with email/password
- JWT access token generation
- Refresh token rotation
- Token validation
- User registration
- Logout (token revocation)

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `AUTH_LISTEN_ADDR` | `0.0.0.0:4002` | Listen address |
| `AUTH_ISSUER` | `apisentinel` | JWT issuer claim |
| `AUTH_TOKEN_TTL_SECONDS` | `300` | Access token TTL (5 min) |
| `AUTH_REFRESH_TTL_SECONDS` | `604800` | Refresh token TTL (7 days) |
| `AUTH_JWT_SECRET` | (required) | JWT signing secret |
| `AUTH_DEFAULT_ADMIN_EMAIL` | (optional) | Auto-create admin if no users |
| `AUTH_DEFAULT_ADMIN_PASSWORD` | (optional) | Password for default admin |
| `ADMIN_SERVICE_URL` | `http://localhost:4001` | Admin service URL (microservices mode) |

## API Endpoints

All endpoints are prefixed with `/auth`:

| Endpoint | Method | Description | Auth Required |
|----------|--------|-------------|---------------|
| `/auth/login` | POST | Login with email/password | No |
| `/auth/refresh` | POST | Refresh access token | Refresh token |
| `/auth/validate` | GET | Validate access token | Bearer token |
| `/auth/logout` | POST | Revoke refresh token | Refresh token |
| `/auth/register` | POST | Register new user | No |

### Login Request/Response

```json
// POST /auth/login
{
  "email": "user@example.com",
  "password": "secret123"
}

// Response
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "a1b2c3d4e5f6...",
  "token_type": "Bearer",
  "expires_in": 300
}
```

### Refresh Request/Response

```json
// POST /auth/refresh
{
  "refresh_token": "a1b2c3d4e5f6..."
}

// Response (new tokens, old refresh token invalidated)
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "x9y8z7w6v5u4...",
  "token_type": "Bearer",
  "expires_in": 300
}
```

## Security Features

- **Password hashing**: Argon2id with secure defaults
- **JWT tokens**: HS256 signed, short-lived (5 min default)
- **Refresh token rotation**: Each refresh invalidates the old token
- **Token hashing**: Refresh tokens stored as SHA-256 hashes
- **Default admin**: Only created when no users exist in database

## Contract Implementations

Auth service uses contracts to communicate with admin service:

| Mode | Implementation | Communication |
|------|----------------|---------------|
| Monolith | `InMemoryUserService` | Direct database |
| Microservices | `HttpUserService` | HTTP to admin `/internal/*` |

```rust
// HTTP implementations (calls admin service)
pub struct HttpUserService { base_url: String, client: Client }
pub struct HttpRefreshTokenService { base_url: String, client: Client }
```

## JWT Claims

```json
{
  "sub": "user-uuid",
  "email": "user@example.com",
  "name": "User Name",
  "iss": "apisentinel",
  "iat": 1706745600,
  "exp": 1706745900
}
```

## Notes
- Runs on port 4002 in microservices mode
- Embedded in gateway on port 4000 in monolith mode
- In monolith mode, uses shared database pool (no HTTP overhead)
- Refresh tokens are single-use (rotation on each refresh)

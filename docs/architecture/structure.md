# Project Structure (Modular Monolith)

This repository implements a **Modular Monolith** architecture that supports both deployment modes:

- **Monolith mode**: Single binary with all services embedded, communicating via in-memory calls.
- **Microservices mode**: Independent service binaries communicating via HTTP.

The design uses a **contracts crate** with trait-based abstractions, allowing seamless switching between deployment modes without code changes.

---

## Top-Level Layout

```
app/                    — Single-binary runtime (monolith)
services/               — Independent service binaries (microservices)
crates/
  contracts/            — Service contracts (traits) and shared types
  common/               — Shared utilities
  observability/        — Logging and metrics
  gateway_core/         — Gateway logic (proxy, middleware, rate limiting)
  auth_core/            — Auth logic (JWT, password hashing)
  admin_core/           — Admin logic (users CRUD, database)
front/                  — React Admin UI
scripts/                — Run helpers
docker-compose.yml      — Local PostgreSQL
docs/                   — Architecture and API documentation
Cargo.toml              — Workspace members
```

---

## Core Architecture: Contracts

The **contracts crate** defines service interfaces as Rust traits:

```rust
#[async_trait]
pub trait UserServiceContract: Send + Sync {
    async fn count(&self) -> ContractResult<i64>;
    async fn find_by_email(&self, email: &str) -> ContractResult<Option<UserWithPassword>>;
    async fn create(&self, email: &str, name: &str, password_hash: &str) -> ContractResult<UserWithPassword>;
    // ...
}
```

Each contract has **two implementations**:

| Implementation | Location | Mode | Communication |
|----------------|----------|------|---------------|
| `InMemoryUserService` | admin_core | Monolith | Direct database calls |
| `HttpUserService` | auth_core | Microservices | HTTP to admin service |

---

## Deployment Modes

### Monolith Mode (Single Binary)

```
┌─────────────────────────────────────────────────────┐
│                 Gateway (:4000)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │   /admin    │  │   /auth     │  │   proxy     │  │
│  │  (embedded) │  │  (embedded) │  │  (upstream) │  │
│  └──────┬──────┘  └──────┬──────┘  └─────────────┘  │
│         │                │                          │
│         └────────┬───────┘                          │
│                  ▼                                  │
│         ┌───────────────┐                           │
│         │   PostgreSQL  │                           │
│         │   (shared)    │                           │
│         └───────────────┘                           │
└─────────────────────────────────────────────────────┘
```

- All services run in one process
- Shared database connection pool
- In-memory service calls (no HTTP overhead)
- Build: `cargo build --package apisentinel-app --all-features`

### Microservices Mode

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Gateway   │────▶│    Auth     │────▶│   Admin     │
│   (:4000)   │     │   (:4002)   │     │   (:4001)   │
│   (proxy)   │     │   (HTTP)    │     │   (DB)      │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                                        ┌──────▼──────┐
                                        │  PostgreSQL │
                                        └─────────────┘
```

- Each service runs independently
- Auth calls Admin via HTTP `/internal/*` endpoints
- Gateway proxies requests to upstream services
- Build: `cargo build --package apisentinel-gateway apisentinel-auth apisentinel-admin`

---

## Internal APIs

Admin exposes internal endpoints for service-to-service communication:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/internal/users/count` | GET | Get user count |
| `/internal/users/by-email/{email}` | GET | Find user by email |
| `/internal/users/{id}` | GET | Get user by ID |
| `/internal/users` | POST | Create user |
| `/internal/refresh-tokens` | POST | Create refresh token |
| `/internal/refresh-tokens/by-hash/{hash}` | GET/DELETE | Manage by hash |
| `/internal/refresh-tokens/{id}` | PUT/DELETE | Update/delete token |

---

## Crate Dependencies

```
contracts (traits, types)
    ▲           ▲
    │           │
admin_core  auth_core
(InMemory)    (HTTP)
    ▲           ▲
    │           │
    └─────┬─────┘
          │
        app (wires implementations)
```

- `contracts` has no dependencies on other crates
- `admin_core` implements contracts with direct DB access
- `auth_core` implements contracts with HTTP client
- `app` wires the appropriate implementation at runtime

---

## Build Modes

| Command | Result |
|---------|--------|
| `cargo build --package apisentinel-app --all-features` | Single binary (monolith) |
| `cargo build --package apisentinel-app --features gateway,admin` | Gateway + Admin only |
| `cargo build --package apisentinel-gateway` | Standalone gateway |
| `cargo build --package apisentinel-auth` | Standalone auth |
| `cargo build --package apisentinel-admin` | Standalone admin |

---

## Why This Works

- **Zero code duplication**: Core logic lives in shared crates
- **Flexible deployment**: Same code, different wiring
- **No runtime overhead in monolith**: In-memory calls, no serialization
- **Clear boundaries**: Contracts enforce API stability
- **Easy testing**: Mock implementations for unit tests

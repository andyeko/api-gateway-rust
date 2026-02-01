# Gateway Service

## Purpose
API Gateway handling routing, reverse proxying, middleware execution, rate limiting, and service embedding.

## Ownership
- Core crate: `crates/gateway_core`
- Microservice: `services/gateway`
- Module (monolith): `app/modules/gateway`

## Responsibilities
- HTTP ingress (single entry point)
- Route-based service embedding or proxying
- Middleware pipeline execution
- Rate limiting
- CORS handling
- WASM/plugin execution (future)

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `GATEWAY_LISTEN_ADDR` | `0.0.0.0:4000` | Listen address |
| `GATEWAY_ADMIN_MODE` | `embedded` | `embedded` or `proxy` |
| `GATEWAY_AUTH_MODE` | `embedded` | `embedded` or `proxy` |
| `GATEWAY_ADMIN_UPSTREAM` | `http://localhost:4001` | Admin service URL (proxy mode) |
| `GATEWAY_AUTH_UPSTREAM` | `http://localhost:4002` | Auth service URL (proxy mode) |

## Route Modes

Each route can be configured as **embedded** or **proxy**:

| Mode | Description | Use Case |
|------|-------------|----------|
| `embedded` | Service router runs in-process | Monolith deployment |
| `proxy` | Requests forwarded to upstream | Microservices deployment |

## Routing

| Path | Embedded Handler | Proxy Target |
|------|------------------|--------------|
| `/admin/*` | `admin_core::build_router()` | `GATEWAY_ADMIN_UPSTREAM` |
| `/auth/*` | `auth_core::build_inner_router()` | `GATEWAY_AUTH_UPSTREAM` |
| `/*` | - | Configurable upstreams |

## Architecture

### Monolith Mode (Embedded)

```
Client → Gateway(:4000) → Embedded Router → Database
                              ↓
                         In-Memory Calls
```

- Admin and Auth routers embedded directly
- Shared database connection pool
- Zero network overhead for internal calls

### Microservices Mode (Proxy)

```
Client → Gateway(:4000) → HTTP Proxy → Auth(:4002) → Admin(:4001) → Database
```

- Gateway proxies requests to upstream services
- Each service runs independently
- Service-to-service communication via HTTP

## Middleware Pipeline

1. **Rate Limiting**: Token bucket per client IP
2. **CORS**: Configurable origin/method/header rules
3. **Request Logging**: Timing and status tracking
4. **WASM Plugins**: Custom middleware (future)

## API

Gateway itself exposes no APIs - it routes to embedded/upstream services:

| Path | Target |
|------|--------|
| `/admin/*` | Admin service (users CRUD) |
| `/auth/*` | Auth service (login, refresh, etc.) |

## Embedding Services

In monolith mode, gateway embeds service routers:

```rust
// app/src/modules/gateway.rs
let user_service = Arc::new(InMemoryUserService::new(pool.clone()));
let token_service = Arc::new(InMemoryRefreshTokenService::new(pool.clone()));

let auth_router = auth_core::server::build_inner_router(
    user_service,
    token_service,
    Arc::new(auth_config),
);
routers.insert("/auth".to_string(), auth_router);
```

## Notes
- Primary entry point in both deployment modes
- In monolith mode, admin and auth share the database pool
- Proxy mode adds latency but enables independent scaling
- Rate limiting applies to all routes

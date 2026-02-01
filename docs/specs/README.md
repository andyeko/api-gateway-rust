# The Sentinel (API Gateway)

## Objective
Build a reverse proxy capable of routing traffic, rate-limiting requests, and executing dynamic middleware with near-zero overhead.

---

## Current Implementation (February 2026)

### Modular Monolith Architecture
- **Contracts crate**: Trait-based service abstractions (`UserServiceContract`, `RefreshTokenServiceContract`)
- **Two implementations per contract**: In-memory (monolith) and HTTP (microservices)
- **Single codebase**: Same code runs as monolith or microservices

### Services
- **Gateway** (:4000): Routing, embedding, proxying, rate limiting
- **Auth** (:4002): JWT authentication, refresh token rotation, Argon2 password hashing
- **Admin** (:4001): Users CRUD, internal APIs, database operations

### Frontend
- React Admin UI with Material-UI
- Login page with JWT authentication
- Users management

---

## Local Development

### Prerequisites
- Rust 2024 edition
- Docker (for PostgreSQL)
- Node.js (for frontend)

### Start PostgreSQL

```bash
docker compose up -d
```

### RustRover Configurations

| Configuration | Mode | Description |
|--------------|------|-------------|
| **App (Single Binary)** | Monolith | All services on :4000 |
| **Monolith + Frontend** | Monolith + UI | Gateway :4000, Vite :5173 |
| **Microservices (Backend)** | Microservices | Admin :4001, Auth :4002, Gateway :4000 |
| **All Services (Compound)** | Microservices + UI | All services + Vite :5173 |

### Command Line

```bash
# Monolith mode
cargo run --package apisentinel-app --features gateway,auth,admin

# Microservices mode
cargo run --package apisentinel-admin &
cargo run --package apisentinel-auth &
cargo run --package apisentinel-gateway &

# Frontend
cd front && npm run dev
```

### Environment Variables

```bash
# Database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/apisentinel

# Gateway
GATEWAY_LISTEN_ADDR=0.0.0.0:4000
GATEWAY_ADMIN_MODE=embedded  # or proxy
GATEWAY_AUTH_MODE=embedded   # or proxy

# Auth
AUTH_LISTEN_ADDR=0.0.0.0:4002
AUTH_JWT_SECRET=your-secret-key
AUTH_DEFAULT_ADMIN_EMAIL=admin@example.com
AUTH_DEFAULT_ADMIN_PASSWORD=admin123

# Admin
ADMIN_BIND_ADDR=0.0.0.0:4001
```

### Test Login

```bash
curl -X POST http://localhost:4000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"admin123"}'
```

---

## Architecture

See [architecture/structure.md](../architecture/structure.md) for detailed diagrams.

### Key Design Decisions

1. **Contracts Crate**: Defines service interfaces as traits, enabling compile-time abstraction
2. **Two Implementations**: `InMemory*` for monolith (direct DB), `Http*` for microservices
3. **Shared Database Pool**: In monolith mode, all services share one connection pool
4. **Internal APIs**: Admin exposes `/internal/*` endpoints for service-to-service calls

---

## Stage 1: The Foundation (Async I/O & Zero-Copy)
The goal is to move bytes from a client to a backend service without unnecessary allocations.

### Requirements
- Implement a basic HTTP server using axum.
- Proxy requests to a hardcoded upstream service (e.g., localhost:8080 -> google.com).

### The Rust Challenge
Ensure Zero-Copy proxying. You should pass the request body as a stream rather than collecting it into a `Vec<u8>` or `String`.

### Concepts to Master
- Tokio runtime (event loop).
- Future-based concurrency.
- Ownership of request/response objects.

---

## Stage 2: Shared State (Thread-Safe Rate Limiting)
In Java, you’d use `ConcurrentHashMap`. In Rust, you must prove to the compiler that your shared state is safe.

### Requirements
- Implement a leaky bucket or fixed-window rate limiter.
- Store request counts per IP address in a global state.

### The Rust Challenge
Use `Arc<Mutex<T>>` or `Arc<RwLock<T>>`. For extra credit, use `DashMap` and compare the performance.

### Concepts to Master
- `Arc` (atomic reference counting) vs. GC.
- Interior mutability pattern.
- `Send` and `Sync` traits.

---

## Stage 3: The Middleware Pipeline (Traits & Polymorphism)
This is where you replicate the flexibility of Python decorators or Java filters.

### Requirements
- Define a `Middleware` trait with a method: `fn process(&self, req: Request) -> Result<Request, Response>`.
- Implement at least three filters: Logging, Auth (API key check), and HeaderInjection.

### The Rust Challenge
Composition. Decide between static dispatch (generics, faster) and dynamic dispatch (`Box<dyn Middleware>`, more flexible). You'll likely hit "Sized" constraint issues here—it's a rite of passage.

### Concepts to Master
- Trait objects and vtables.
- Associated types.
- Error handling with `thiserror` or `anyhow`.

---

## Stage 4: Hot-Swappable Logic (WASM or Plugins)
Senior level: load new routing logic without restarting the process.

### Requirements
- Integrate wasmtime or wasmer.
- Allow the gateway to load a `.wasm` file that handles specific request transformations.

### The Rust Challenge
Managing the boundary between the "Host" (your gateway) and the "Guest" (WASM). You'll deal with memory isolation and passing complex data types across the FFI (Foreign Function Interface) boundary.

### Concepts to Master
- WASM guest/host interaction.
- Serialization/deserialization (`serde`) performance.
- Unsafe code (if dealing with raw pointers for FFI).

---

## Recommended Crates
| Component       | Recommended Crate | Why? |
|----------------|-------------------|------|
| HTTP Engine    | axum             | Low-level, fast, the industry standard for proxies. |
| Async Runtime  | tokio             | The de-facto standard for production Rust. |
| Serialization  | serde             | The most powerful serialization framework in existence. |
| Shared State   | dashmap           | High-concurrency alternative to `Mutex<HashMap>`. |
| WASM           | wasmtime          | Extremely secure and backed by the Bytecode Alliance. |
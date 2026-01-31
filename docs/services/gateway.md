# Gateway Service

## Purpose
Handles reverse proxying, routing, middleware execution, and rate limiting.

## Ownership
- Core crate: crates/gateway_core
- Microservice: services/gateway
- Module (single app): app/modules/gateway

## Responsibilities
- HTTP ingress
- Upstream routing
- Middleware pipeline
- Rate limiting
- WASM/plugin execution (future)

## Configuration
- Placeholder (define in configs/ when available)

## Notes
- Current implementation is a stubbed in-process flow (no HTTP listener yet).
- Build and run independently or via the app module.

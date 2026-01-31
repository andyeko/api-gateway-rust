# Project Structure (Hybrid: Modules + Microservices)

This repository supports **both** deployment modes:

- **Single-app mode**: build one binary that runs multiple services as modules.
- **Microservices mode**: build and run each service independently.

The design centers around shared core crates that are reused by both modes.

---

## Top-Level Layout

- app/ — single-binary runtime (modules)
- services/ — independent service binaries
- crates/ — shared core libraries
- front/ — React Admin UI
- scripts/ — run helpers (single app or microservices)
- docker-compose.yml — local Postgres
- configs/ — config files per service
- docs/ — design and architecture notes
- tests/ — cross-service integration tests
- Cargo.toml — workspace members

---

## Core Principle

Each service has a **core crate** that holds all business logic and is reused by:

1. The single-app module adapters (app/modules)
2. The standalone microservice binaries (services/*)

This avoids code duplication while allowing flexible deployment.

---

## Suggested Structure

### Single-App Runtime

- app/
  - src/main.rs — entry point; loads modules via feature flags
  - src/modules/ — adapters that wire core crates into a single process

### Microservices

- services/
  - gateway/
  - auth/
  - admin/

Each service folder contains a standalone binary that depends on its core crate.

### Shared Crates

- crates/common/ — shared types, errors, config helpers
- crates/observability/ — shared observability hooks
- crates/gateway_core/ — gateway logic (proxy, middleware, rate limiting, WASM)
- crates/auth_core/ — auth logic
- crates/admin_core/ — admin logic

---

## Build Modes

- **Single app**: build the app binary with feature flags for enabled modules.
- **Microservices**: build each service independently from services/*.

---

## Why This Works

- Shared code lives in core crates.
- Modules and services are thin wrappers.
- Deployment is flexible without branching the codebase.

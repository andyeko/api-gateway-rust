# Admin Service

## Purpose
Administrative APIs and operational controls.

## Ownership
- Core crate: crates/admin_core
- Microservice: services/admin
- Module (single app): app/modules/admin

## Responsibilities
- Admin endpoints
- Service management
- Users CRUD (current)
- Health and status

## Configuration
- Environment
	- DATABASE_URL (default: postgres://postgres:postgres@localhost:5432/apisentinel)
	- ADMIN_BIND_ADDR (default: 0.0.0.0:4001)

## API (Users CRUD)
- GET /users
	- Supports react-admin pagination via _start and _end
- GET /users/:id
- POST /users
- PUT /users/:id
- DELETE /users/:id

## Notes
- Build and run independently or via the app module.

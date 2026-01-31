# Auth Service

## Purpose
Provides authentication and authorization for the platform.

## Ownership
- Core crate: crates/auth_core
- Microservice: services/auth
- Module (single app): app/modules/auth

## Responsibilities
- API key validation (planned)
- Token verification/issuance (planned)
- User/session checks (planned)

## Configuration
- Environment
	- AUTH_ISSUER (default: apisentinel)

## Notes
- Build and run independently or via the app module.

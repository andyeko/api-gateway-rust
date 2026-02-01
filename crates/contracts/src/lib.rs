//! Contracts crate - Shared traits and types for service-to-service communication
//! 
//! This crate provides the abstraction layer that enables the modular monolith pattern:
//! - When running as a single binary: use in-memory implementations (no network overhead)
//! - When running as separate services: use HTTP implementations
//!
//! All service communication is defined through traits, allowing different backends.

pub mod types;
pub mod user;
pub mod token;

pub use types::*;
pub use user::UserServiceContract;
pub use token::RefreshTokenServiceContract;

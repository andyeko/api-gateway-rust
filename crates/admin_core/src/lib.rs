pub mod config;
pub mod contract_impl;
pub mod db;
pub mod handlers;
pub mod internal_handlers;
pub mod models;
pub mod server;
pub mod service;
pub mod user_service;

// Re-export types for use by the app
pub use db::DbPool;

// Re-export contract implementations for monolith mode
pub use contract_impl::{InMemoryRefreshTokenService, InMemoryUserService};

// Re-export the old services for backward compatibility
pub use user_service::{RefreshTokenService, UserService, UserWithPassword, UserInfo};

pub async fn run() -> anyhow::Result<()> {
    let config = config::AdminConfig::default();
    service::run(&config).await
}

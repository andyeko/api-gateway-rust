pub mod config;
pub mod handlers;
pub mod http_client;
pub mod models;
pub mod server;
pub mod service;
pub mod token;

// Re-export HTTP client implementations for microservice mode
pub use http_client::{HttpRefreshTokenService, HttpUserService};

// Re-export contracts for convenience
pub use contracts::{RefreshTokenServiceContract, UserServiceContract, UserWithPassword};

pub use config::AuthConfig;

pub async fn run() -> anyhow::Result<()> {
    let config = config::AuthConfig::default();
    service::run(&config).await
}

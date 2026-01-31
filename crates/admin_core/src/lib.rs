pub mod config;
pub mod db;
pub mod handlers;
pub mod models;
pub mod server;
pub mod service;
pub mod user_service;

pub use user_service::{RefreshTokenService, UserService, UserWithPassword, UserInfo};

pub async fn run() -> anyhow::Result<()> {
    let config = config::AdminConfig::default();
    service::run(&config).await
}

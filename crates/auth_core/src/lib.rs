pub mod config;
pub mod handlers;
pub mod models;
pub mod server;
pub mod service;
pub mod token;

pub use config::AuthConfig;

pub async fn run() -> anyhow::Result<()> {
    let config = config::AuthConfig::default();
    service::run(&config).await
}

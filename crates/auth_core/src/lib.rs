pub mod config;
pub mod service;

pub async fn run() -> anyhow::Result<()> {
    let config = config::AuthConfig::default();
    service::run(&config).await
}

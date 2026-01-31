pub mod config;
pub mod db;
pub mod handlers;
pub mod models;
pub mod server;
pub mod service;

pub async fn run() -> anyhow::Result<()> {
    let config = config::AdminConfig::default();
    service::run(&config).await
}

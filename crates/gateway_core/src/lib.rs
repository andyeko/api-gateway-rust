pub mod config;
pub mod middleware;
pub mod proxy;
pub mod rate_limit;
pub mod server;
pub mod types;
pub mod wasm;

pub async fn run() -> anyhow::Result<()> {
    let config = config::GatewayConfig::default();
    wasm::init();
    server::run(&config).await
}

pub async fn run_with_admin_router(admin_router: axum::Router) -> anyhow::Result<()> {
    let config = config::GatewayConfig::default();
    wasm::init();
    server::run_with_admin_router(&config, Some(admin_router)).await
}

use std::collections::HashMap;

pub mod config;
pub mod middleware;
pub mod proxy;
pub mod rate_limit;
pub mod server;
pub mod types;
pub mod wasm;

pub use config::{GatewayConfig, RouteConfig, RouteMode};
pub use middleware::set_jwt_secret;

/// Run gateway with default configuration (all routes proxied based on env config)
pub async fn run() -> anyhow::Result<()> {
    let config = config::GatewayConfig::default();
    wasm::init();
    server::run(&config).await
}

/// Run gateway with embedded routers for specific routes
/// Routes not in `routers` map will be proxied based on config
pub async fn run_with_routers(routers: HashMap<String, axum::Router>) -> anyhow::Result<()> {
    let config = config::GatewayConfig::default();
    wasm::init();
    server::run_with_routers(&config, routers).await
}

/// Run gateway with custom config and embedded routers
pub async fn run_with_config_and_routers(
    config: GatewayConfig,
    routers: HashMap<String, axum::Router>,
) -> anyhow::Result<()> {
    wasm::init();
    server::run_with_routers(&config, routers).await
}

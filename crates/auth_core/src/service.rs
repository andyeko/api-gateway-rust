use std::sync::Arc;

use crate::config::AuthConfig;
use crate::http_client::{HttpRefreshTokenService, HttpUserService};

/// Run the auth service in standalone (microservice) mode
/// Uses HTTP to communicate with admin service
pub async fn run(config: &AuthConfig) -> anyhow::Result<()> {
    println!("auth service starting on {}", config.listen_addr);
    println!("  mode: standalone (HTTP client)");
    println!("  issuer: {}", config.issuer);
    println!("  token TTL: {}s", config.token_ttl_seconds);
    println!("  admin service: {}", config.admin_service_url);

    if config.default_admin_email.is_some() {
        println!("  default admin: enabled (active only when no users exist)");
    }

    // Create HTTP-based service implementations
    let user_service = Arc::new(HttpUserService::new(&config.admin_service_url));
    let token_service = Arc::new(HttpRefreshTokenService::new(&config.admin_service_url));

    let config = Arc::new(config.clone());
    crate::server::run(&config.listen_addr, user_service, token_service, config.clone())
        .await
        .map_err(|e| anyhow::anyhow!("server error: {}", e))
}

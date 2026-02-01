use std::sync::Arc;

use admin_core::{DbPool, InMemoryRefreshTokenService, InMemoryUserService};
use auth_core::AuthConfig;

/// Start auth module in embedded (monolith) mode
/// Uses in-memory implementations - no HTTP calls to admin service
pub async fn start(pool: DbPool) {
    let config = AuthConfig::default();

    println!("auth module starting on {} (embedded mode)", config.listen_addr);
    println!("  mode: embedded (in-memory)");
    println!("  issuer: {}", config.issuer);
    println!("  token TTL: {}s", config.token_ttl_seconds);

    if config.default_admin_email.is_some() {
        println!("  default admin: enabled (active only when no users exist)");
    }

    // Create in-memory service implementations (direct database access)
    let user_service = Arc::new(InMemoryUserService::new(pool.clone()));
    let token_service = Arc::new(InMemoryRefreshTokenService::new(pool));

    let config = Arc::new(config);
    if let Err(err) = auth_core::server::run(
        &config.listen_addr,
        user_service,
        token_service,
        config.clone(),
    )
    .await
    {
        eprintln!("auth module error: {err}");
    }
}

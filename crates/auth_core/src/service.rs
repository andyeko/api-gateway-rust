use std::sync::Arc;

use crate::config::AuthConfig;

pub async fn run(config: &AuthConfig) -> anyhow::Result<()> {
    println!("auth service starting on {}", config.listen_addr);
    println!("  issuer: {}", config.issuer);
    println!("  token TTL: {}s", config.token_ttl_seconds);

    if config.default_admin_email.is_some() {
        println!("  default admin: enabled (active only when no users exist)");
    }

    // Use admin_core for database connection
    let admin_config = admin_core::config::AdminConfig::default();
    let pool = admin_core::db::create_pool(&admin_config.database_url).await?;

    // Run migrations (uses admin_core's migrations which include auth tables)
    admin_core::db::migrate(&pool).await?;
    println!("  database: connected and migrated");

    // Start server
    let config = Arc::new(config.clone());
    crate::server::run(&config.listen_addr, pool, config.clone())
        .await
        .map_err(|e| anyhow::anyhow!("server error: {}", e))
}

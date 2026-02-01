use std::collections::HashMap;
use std::sync::Arc;

/// Start gateway with embedded modules based on feature flags and config
pub async fn start(pool: Option<admin_core::DbPool>) {
    let config = gateway_core::GatewayConfig::default();
    let mut routers: HashMap<String, axum::Router> = HashMap::new();

    // If admin feature is enabled and route is configured as embedded, add admin router
    #[cfg(feature = "admin")]
    {
        if let Some(ref pool) = pool {
            if !config.is_proxy("/admin") {
                let router = admin_core::server::build_router(pool.clone());
                routers.insert("/admin".to_string(), router);
            }
        }
    }

    // If auth feature is enabled and route is configured as embedded, add auth router
    #[cfg(feature = "auth")]
    {
        if let Some(ref pool) = pool {
            if !config.is_proxy("/auth") {
                let auth_config = auth_core::AuthConfig::default();

                // Create in-memory service implementations (direct database access)
                let user_service = Arc::new(admin_core::InMemoryUserService::new(pool.clone()));
                let token_service = Arc::new(admin_core::InMemoryRefreshTokenService::new(pool.clone()));

                let router = auth_core::server::build_inner_router(
                    user_service,
                    token_service,
                    Arc::new(auth_config),
                );
                routers.insert("/auth".to_string(), router);
            }
        }
    }

    // Suppress unused variable warning when neither admin nor auth feature is enabled
    #[cfg(not(any(feature = "admin", feature = "auth")))]
    let _ = pool;

    if let Err(err) = gateway_core::run_with_config_and_routers(config, routers).await {
        eprintln!("gateway module error: {err}");
    }
}

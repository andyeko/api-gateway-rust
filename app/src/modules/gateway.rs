use std::collections::HashMap;

/// Start gateway with embedded modules based on feature flags and config
pub async fn start() {
    let config = gateway_core::GatewayConfig::default();
    let mut routers: HashMap<String, axum::Router> = HashMap::new();

    // If admin feature is enabled and route is configured as embedded, add admin router
    #[cfg(feature = "admin")]
    {
        if !config.is_proxy("/admin") {
            match setup_admin_router().await {
                Ok(router) => {
                    routers.insert("/admin".to_string(), router);
                }
                Err(err) => {
                    eprintln!("failed to setup admin router: {err}");
                }
            }
        }
    }

    // Future: add auth router here when auth_core supports it
    // #[cfg(feature = "auth")]
    // {
    //     if !config.is_proxy("/auth") {
    //         routers.insert("/auth".to_string(), auth_core::build_router());
    //     }
    // }

    if let Err(err) = gateway_core::run_with_config_and_routers(config, routers).await {
        eprintln!("gateway module error: {err}");
    }
}

#[cfg(feature = "admin")]
async fn setup_admin_router() -> anyhow::Result<axum::Router> {
    let admin_config = admin_core::config::AdminConfig::default();

    let pool = admin_core::db::create_pool(&admin_config.database_url).await?;
    admin_core::db::migrate(&pool).await?;

    Ok(admin_core::server::build_router(pool))
}

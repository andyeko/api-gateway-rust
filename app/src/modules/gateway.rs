#[cfg(not(feature = "admin"))]
pub async fn start() {
    if let Err(err) = gateway_core::run().await {
        eprintln!("gateway module error: {err}");
    }
}

#[cfg(feature = "admin")]
pub async fn start() {
    let admin_config = admin_core::config::AdminConfig::default();

    let pool = match admin_core::db::create_pool(&admin_config.database_url).await {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("gateway admin setup error: {err}");
            return;
        }
    };

    if let Err(err) = admin_core::db::migrate(&pool).await {
        eprintln!("gateway admin migrations error: {err}");
        return;
    }

    let admin_router = admin_core::server::build_router(pool);
    if let Err(err) = gateway_core::run_with_admin_router(admin_router).await {
        eprintln!("gateway module error: {err}");
    }
}

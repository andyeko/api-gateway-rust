mod modules;

#[tokio::main]
async fn main() {
    println!("apisentinel app");
    println!("  mode: modular monolith");

    // Initialize shared database pool once for all modules
    #[cfg(any(feature = "admin", feature = "auth"))]
    let pool = {
        let admin_config = admin_core::config::AdminConfig::default();
        println!("  database: connecting to {}", admin_config.database_url);

        let pool = match admin_core::db::create_pool(&admin_config.database_url).await {
            Ok(pool) => pool,
            Err(err) => {
                eprintln!("failed to connect to database: {err}");
                return;
            }
        };

        if let Err(err) = admin_core::db::migrate(&pool).await {
            eprintln!("failed to run migrations: {err}");
            return;
        }
        println!("  database: connected and migrated");
        pool
    };

    // Set JWT secret for gateway to validate tokens
    #[cfg(feature = "gateway")]
    {
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        gateway_core::set_jwt_secret(jwt_secret);
    }

    let handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    // Gateway is the main entry point - it handles routing and can embed other modules
    #[cfg(feature = "gateway")]
    {
        #[cfg(any(feature = "admin", feature = "auth"))]
        let gateway_pool = Some(pool.clone());
        #[cfg(not(any(feature = "admin", feature = "auth")))]
        let gateway_pool: Option<admin_core::DbPool> = None;

        let handle = tokio::spawn(async move {
            modules::gateway::start(gateway_pool).await;
        });

        // Gateway is the only service in monolith mode, wait for it
        let _ = handle.await;
        return;
    }

    // Auth runs as a separate service only when gateway is not enabled
    // (otherwise auth is embedded in gateway)
    #[cfg(all(feature = "auth", not(feature = "gateway")))]
    {
        let auth_pool = pool.clone();
        let handle = tokio::spawn(async move {
            modules::auth::start(auth_pool).await;
        });
        let _ = handle.await;
        return;
    }

    // Admin runs standalone only when gateway is not enabled
    // (otherwise admin is embedded in gateway or proxied by gateway)
    #[cfg(all(feature = "admin", not(feature = "gateway")))]
    {
        let admin_pool = pool.clone();
        let handle = tokio::spawn(async move {
            modules::admin::start(admin_pool).await;
        });
        let _ = handle.await;
        return;
    }

    if handles.is_empty() {
        eprintln!("No modules enabled. Use --features to enable: gateway, auth, admin");
    }
}

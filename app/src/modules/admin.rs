use admin_core::{config::AdminConfig, DbPool};

/// Start admin module in standalone mode
/// Uses the shared database pool from main
pub async fn start(pool: DbPool) {
    let config = AdminConfig::default();
    println!("admin module starting on {}", config.bind_addr);

    if let Err(err) = admin_core::server::run(&config.bind_addr, pool).await {
        eprintln!("admin module error: {err}");
    }
}

use anyhow::Context;

use crate::config::AdminConfig;
use crate::db;
use crate::server;

pub async fn run(config: &AdminConfig) -> anyhow::Result<()> {
    println!("admin service running on {}", config.bind_addr);

    let pool = db::create_pool(&config.database_url).await.context("create pool")?;
    db::migrate(&pool).await.context("run migrations")?;

    server::run(&config.bind_addr, pool)
        .await
        .context("start http server")?;

    Ok(())
}

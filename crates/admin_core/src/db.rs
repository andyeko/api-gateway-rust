use anyhow::Context;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub type DbPool = PgPool;

pub async fn create_pool(database_url: &str) -> anyhow::Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("create database pool")?;
    Ok(pool)
}

pub async fn migrate(pool: &DbPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await
    .context("run migrations")?;

    Ok(())
}

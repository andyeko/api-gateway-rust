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
    // Add password_hash column to users table for authentication
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            password_hash TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await
    .context("create users table")?;

    // Add password_hash column if it doesn't exist (for existing tables)
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'users' AND column_name = 'password_hash'
            ) THEN
                ALTER TABLE users ADD COLUMN password_hash TEXT;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await
    .context("add password_hash column")?;

    // Create refresh tokens table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            token_hash TEXT NOT NULL UNIQUE,
            expires_at TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await
    .context("create refresh_tokens table")?;

    // Index for faster token lookup
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token_hash 
        ON refresh_tokens(token_hash);
        "#,
    )
    .execute(pool)
    .await
    .context("create refresh_tokens index")?;

    Ok(())
}

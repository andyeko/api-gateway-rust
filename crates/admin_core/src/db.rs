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
    // Create role enum type
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_role') THEN
                CREATE TYPE user_role AS ENUM ('SUPER_ADMIN', 'ADMIN', 'SUPERVISOR', 'USER');
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await
    .context("create user_role enum")?;

    // Create organisations table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS organisations (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await
    .context("create organisations table")?;

    // Create users table with organisation_id and role
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            organisation_id UUID REFERENCES organisations(id) ON DELETE SET NULL,
            email TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            password_hash TEXT,
            role user_role NOT NULL DEFAULT 'USER',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await
    .context("create users table")?;

    // Add new columns if they don't exist (for existing tables)
    sqlx::query(
        r#"
        DO $$
        BEGIN
            -- Add password_hash column
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'users' AND column_name = 'password_hash'
            ) THEN
                ALTER TABLE users ADD COLUMN password_hash TEXT;
            END IF;
            
            -- Add organisation_id column
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'users' AND column_name = 'organisation_id'
            ) THEN
                ALTER TABLE users ADD COLUMN organisation_id UUID REFERENCES organisations(id) ON DELETE SET NULL;
            END IF;
            
            -- Add role column
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'users' AND column_name = 'role'
            ) THEN
                ALTER TABLE users ADD COLUMN role user_role NOT NULL DEFAULT 'USER';
            END IF;
            
            -- Add updated_at column
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'users' AND column_name = 'updated_at'
            ) THEN
                ALTER TABLE users ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await
    .context("add new columns to users")?;

    // Create refresh tokens table for auth
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            organisation_id UUID REFERENCES organisations(id) ON DELETE CASCADE,
            token_hash TEXT NOT NULL UNIQUE,
            expires_at TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await
    .context("create refresh_tokens table")?;

    // Add organisation_id to refresh_tokens if it doesn't exist
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'refresh_tokens' AND column_name = 'organisation_id'
            ) THEN
                ALTER TABLE refresh_tokens ADD COLUMN organisation_id UUID REFERENCES organisations(id) ON DELETE CASCADE;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await
    .context("add organisation_id to refresh_tokens")?;

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

    // Index for organisation lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_users_organisation_id 
        ON users(organisation_id);
        "#,
    )
    .execute(pool)
    .await
    .context("create users organisation index")?;

    Ok(())
}

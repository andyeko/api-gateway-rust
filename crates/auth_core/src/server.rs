use std::sync::Arc;

use admin_core::{RefreshTokenService, UserService};
use admin_core::db::DbPool;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::config::AuthConfig;
use crate::handlers::{login, logout, refresh, register, validate, AppState};

pub async fn run(bind_addr: &str, pool: DbPool, config: Arc<AuthConfig>) -> Result<(), std::io::Error> {
    let app = build_router(pool, config);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await
}

pub fn build_router(pool: DbPool, config: Arc<AuthConfig>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let user_service = UserService::new(pool.clone());
    let token_service = RefreshTokenService::new(pool);

    let state = AppState {
        user_service,
        token_service,
        config,
    };

    // All routes under /auth prefix
    let auth_routes = Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/validate", get(validate))
        .route("/logout", post(logout))
        .route("/register", post(register));

    Router::new()
        .nest("/auth", auth_routes)
        .with_state(state)
        .layer(cors)
}

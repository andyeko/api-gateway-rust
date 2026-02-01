use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use contracts::{RefreshTokenServiceContract, UserServiceContract};

use crate::config::AuthConfig;
use crate::handlers::{login, logout, refresh, register, validate, AppState};

/// Run the auth server with the given service implementations
/// 
/// This allows running with different backends:
/// - In-memory implementations for monolith mode
/// - HTTP implementations for microservice mode
pub async fn run(
    bind_addr: &str,
    user_service: Arc<dyn UserServiceContract>,
    token_service: Arc<dyn RefreshTokenServiceContract>,
    config: Arc<AuthConfig>,
) -> Result<(), std::io::Error> {
    let app = build_router(user_service, token_service, config);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await
}

/// Build the auth router with the given service implementations
/// This router includes the /auth prefix - use for standalone mode
pub fn build_router(
    user_service: Arc<dyn UserServiceContract>,
    token_service: Arc<dyn RefreshTokenServiceContract>,
    config: Arc<AuthConfig>,
) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let inner = build_inner_router(user_service, token_service, config);

    Router::new()
        .nest("/auth", inner)
        .layer(cors)
}

/// Build the inner auth router WITHOUT the /auth prefix
/// Use this when embedding in gateway (gateway will nest under /auth)
pub fn build_inner_router(
    user_service: Arc<dyn UserServiceContract>,
    token_service: Arc<dyn RefreshTokenServiceContract>,
    config: Arc<AuthConfig>,
) -> Router {
    let state = AppState {
        user_service,
        token_service,
        config,
    };

    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/validate", get(validate))
        .route("/logout", post(logout))
        .route("/register", post(register))
        .with_state(state)
}

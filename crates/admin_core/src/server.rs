use axum::Router;
use axum::routing::{delete, get, post, put};
use tower_http::cors::{Any, CorsLayer};

use crate::db::DbPool;
use crate::handlers::{
    AppState,
    // Organisation handlers
    create_organisation, delete_organisation, get_organisation, list_organisations, update_organisation,
    // User handlers
    create_user, delete_user, get_user, list_users, update_user,
};
use crate::internal_handlers::{
    create_refresh_token, create_user_internal, delete_refresh_token,
    delete_refresh_token_by_hash, get_refresh_token_by_hash, get_user_by_email,
    get_user_by_id_internal, get_user_count, update_refresh_token,
};

pub async fn run(bind_addr: &str, pool: DbPool) -> Result<(), std::io::Error> {
    let app = build_router(pool);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await
}

pub fn build_router(pool: DbPool) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState { pool };

    // Internal routes for service-to-service communication
    let internal_routes = Router::new()
        // User endpoints
        .route("/users/count", get(get_user_count))
        .route("/users/by-email/{email}", get(get_user_by_email))
        .route("/users/{id}", get(get_user_by_id_internal))
        .route("/users", post(create_user_internal))
        // Refresh token endpoints
        .route("/refresh-tokens", post(create_refresh_token))
        .route("/refresh-tokens/by-hash/{hash}", get(get_refresh_token_by_hash))
        .route("/refresh-tokens/by-hash/{hash}", delete(delete_refresh_token_by_hash))
        .route("/refresh-tokens/{id}", put(update_refresh_token))
        .route("/refresh-tokens/{id}", delete(delete_refresh_token));

    Router::new()
        // Organisation routes
        .route("/organisations", get(list_organisations).post(create_organisation))
        .route(
            "/organisations/{id}",
            get(get_organisation).put(update_organisation).delete(delete_organisation),
        )
        // User routes
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user).put(update_user).delete(delete_user),
        )
        .nest("/internal", internal_routes)
        .with_state(state)
        .layer(cors)
}

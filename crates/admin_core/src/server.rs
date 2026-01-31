use axum::routing::get;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::db::DbPool;
use crate::handlers::{
    create_user, delete_user, get_user, list_users, update_user, AppState,
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

    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", get(get_user).put(update_user).delete(delete_user))
        .with_state(AppState { pool })
        .layer(cors)
}

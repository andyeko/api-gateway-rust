use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::{CreateUser, UpdateUser, User};

#[derive(Debug, Default, serde::Deserialize)]
pub struct ListQuery {
    pub _start: Option<i64>,
    pub _end: Option<i64>,
    pub _sort: Option<String>,
    pub _order: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
}

pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let start = query._start.unwrap_or(0).max(0);
    let end = query._end.unwrap_or(start + 25).max(start + 1);
    let limit = (end - start) as i64;

    let total: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|err| {
            eprintln!("list_users count error: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let users = sqlx::query_as::<_, User>(
        "SELECT id, email, name, created_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(start)
    .fetch_all(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("list_users fetch error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut headers = HeaderMap::new();
    let content_range = format!("users {}-{}/{}", start, end.saturating_sub(1), total);
    headers.insert(
        "Content-Range",
        HeaderValue::from_str(&content_range).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    headers.insert(
        "X-Total-Count",
        HeaderValue::from_str(&total.to_string()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    headers.insert(
        "Access-Control-Expose-Headers",
        HeaderValue::from_static("Content-Range, X-Total-Count"),
    );

    Ok((headers, Json(users)))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user =
        sqlx::query_as::<_, User>("SELECT id, email, name, created_at FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|err| {
                eprintln!("get_user error: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = Uuid::new_v4();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, name) VALUES ($1, $2, $3) RETURNING id, email, name, created_at",
    )
    .bind(id)
    .bind(&payload.email)
    .bind(&payload.name)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("create_user error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET email = COALESCE($1, email), name = COALESCE($2, name) WHERE id = $3 RETURNING id, email, name, created_at",
    )
    .bind(&payload.email)
    .bind(&payload.name)
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("update_user error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "DELETE FROM users WHERE id = $1 RETURNING id, email, name, created_at",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("delete_user error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

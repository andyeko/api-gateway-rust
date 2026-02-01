use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use uuid::Uuid;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use crate::db::DbPool;
use crate::models::{
    CreateOrganisation, CreateUser, Organisation, Role, UpdateOrganisation, UpdateUser, User,
};

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

/// Extract organisation_id from request headers (set by gateway)
fn get_org_id_from_headers(headers: &HeaderMap) -> Option<Uuid> {
    headers
        .get("x-organisation-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
}

/// Extract user role from request headers (set by gateway)
fn get_role_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-user-role")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Check if user is a super admin (can access all organisations)
fn is_super_admin(headers: &HeaderMap) -> bool {
    get_role_from_headers(headers)
        .map(|r| r == "SUPER_ADMIN")
        .unwrap_or(false)
}

// ============ Organisation Handlers ============

pub async fn list_organisations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let start = query._start.unwrap_or(0).max(0);
    let end = query._end.unwrap_or(start + 25).max(start + 1);
    let limit = (end - start) as i64;

    // Super admins can see all organisations, others only see their own
    let org_id = if is_super_admin(&headers) {
        None
    } else {
        get_org_id_from_headers(&headers)
    };

    let (total, orgs) = if let Some(org_id) = org_id {
        let total: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM organisations WHERE id = $1")
            .bind(org_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|err| {
                eprintln!("list_organisations count error: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let orgs = sqlx::query_as::<_, Organisation>(
            "SELECT id, name, slug, created_at, updated_at FROM organisations WHERE id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(org_id)
        .bind(limit)
        .bind(start)
        .fetch_all(&state.pool)
        .await
        .map_err(|err| {
            eprintln!("list_organisations fetch error: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        (total, orgs)
    } else {
        let total: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM organisations")
            .fetch_one(&state.pool)
            .await
            .map_err(|err| {
                eprintln!("list_organisations count error: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let orgs = sqlx::query_as::<_, Organisation>(
            "SELECT id, name, slug, created_at, updated_at FROM organisations ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(start)
        .fetch_all(&state.pool)
        .await
        .map_err(|err| {
            eprintln!("list_organisations fetch error: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        (total, orgs)
    };

    let mut response_headers = HeaderMap::new();
    let content_range = format!("organisations {}-{}/{}", start, end.saturating_sub(1), total);
    response_headers.insert(
        "Content-Range",
        HeaderValue::from_str(&content_range).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    response_headers.insert(
        "X-Total-Count",
        HeaderValue::from_str(&total.to_string()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    response_headers.insert(
        "Access-Control-Expose-Headers",
        HeaderValue::from_static("Content-Range, X-Total-Count"),
    );

    Ok((response_headers, Json(orgs)))
}

pub async fn get_organisation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check access: super admins can access any org, others only their own
    if !is_super_admin(&headers) {
        if let Some(org_id) = get_org_id_from_headers(&headers) {
            if org_id != id {
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    let org = sqlx::query_as::<_, Organisation>(
        "SELECT id, name, slug, created_at, updated_at FROM organisations WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("get_organisation error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match org {
        Some(org) => Ok(Json(org)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_organisation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateOrganisation>,
) -> Result<impl IntoResponse, StatusCode> {
    // Only super admins can create organisations
    if !is_super_admin(&headers) {
        return Err(StatusCode::FORBIDDEN);
    }

    let id = Uuid::new_v4();

    let org = sqlx::query_as::<_, Organisation>(
        "INSERT INTO organisations (id, name, slug) VALUES ($1, $2, $3) RETURNING id, name, slug, created_at, updated_at",
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.slug)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("create_organisation error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(org)))
}

pub async fn update_organisation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrganisation>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check access: super admins can update any org, admins only their own
    if !is_super_admin(&headers) {
        if let Some(org_id) = get_org_id_from_headers(&headers) {
            if org_id != id {
                return Err(StatusCode::FORBIDDEN);
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let org = sqlx::query_as::<_, Organisation>(
        "UPDATE organisations SET name = COALESCE($1, name), slug = COALESCE($2, slug), updated_at = NOW() WHERE id = $3 RETURNING id, name, slug, created_at, updated_at",
    )
    .bind(&payload.name)
    .bind(&payload.slug)
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("update_organisation error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match org {
        Some(org) => Ok(Json(org)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_organisation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Only super admins can delete organisations
    if !is_super_admin(&headers) {
        return Err(StatusCode::FORBIDDEN);
    }

    let org = sqlx::query_as::<_, Organisation>(
        "DELETE FROM organisations WHERE id = $1 RETURNING id, name, slug, created_at, updated_at",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("delete_organisation error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match org {
        Some(org) => Ok(Json(org)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ============ User Handlers ============

pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let start = query._start.unwrap_or(0).max(0);
    let end = query._end.unwrap_or(start + 25).max(start + 1);
    let limit = (end - start) as i64;

    // Super admins can see all users, others only see users in their organisation
    let org_id = if is_super_admin(&headers) {
        None
    } else {
        get_org_id_from_headers(&headers)
    };

    let (total, users) = if let Some(org_id) = org_id {
        let total: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE organisation_id = $1")
            .bind(org_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|err| {
                eprintln!("list_users count error: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let users = sqlx::query_as::<_, User>(
            "SELECT id, organisation_id, email, name, role, created_at, updated_at FROM users WHERE organisation_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(org_id)
        .bind(limit)
        .bind(start)
        .fetch_all(&state.pool)
        .await
        .map_err(|err| {
            eprintln!("list_users fetch error: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        (total, users)
    } else {
        let total: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&state.pool)
            .await
            .map_err(|err| {
                eprintln!("list_users count error: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let users = sqlx::query_as::<_, User>(
            "SELECT id, organisation_id, email, name, role, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(start)
        .fetch_all(&state.pool)
        .await
        .map_err(|err| {
            eprintln!("list_users fetch error: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        (total, users)
    };

    let mut response_headers = HeaderMap::new();
    let content_range = format!("users {}-{}/{}", start, end.saturating_sub(1), total);
    response_headers.insert(
        "Content-Range",
        HeaderValue::from_str(&content_range).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    response_headers.insert(
        "X-Total-Count",
        HeaderValue::from_str(&total.to_string()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    response_headers.insert(
        "Access-Control-Expose-Headers",
        HeaderValue::from_static("Content-Range, X-Total-Count"),
    );

    Ok((response_headers, Json(users)))
}

pub async fn get_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, organisation_id, email, name, role, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("get_user error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match user {
        Some(user) => {
            // Check access: super admins can see all, others only their org
            if !is_super_admin(&headers) {
                if let Some(caller_org) = get_org_id_from_headers(&headers) {
                    if user.organisation_id != Some(caller_org) {
                        return Err(StatusCode::FORBIDDEN);
                    }
                } else {
                    return Err(StatusCode::FORBIDDEN);
                }
            }
            Ok(Json(user))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = Uuid::new_v4();
    let role = payload.role.unwrap_or(Role::User);

    // Determine organisation_id: super admins can specify any, others use their own
    let organisation_id = if is_super_admin(&headers) {
        payload.organisation_id
    } else {
        // Non-super admins can only create users in their own organisation
        let caller_org = get_org_id_from_headers(&headers).ok_or(StatusCode::FORBIDDEN)?;
        if let Some(req_org) = payload.organisation_id {
            if req_org != caller_org {
                return Err(StatusCode::FORBIDDEN);
            }
        }
        Some(caller_org)
    };

    // Hash password if provided
    let password_hash = if let Some(ref password) = payload.password {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Some(
            argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|err| {
                    eprintln!("create_user password hash error: {err}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .to_string(),
        )
    } else {
        None
    };

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, organisation_id, email, name, password_hash, role) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, organisation_id, email, name, role, created_at, updated_at",
    )
    .bind(id)
    .bind(organisation_id)
    .bind(&payload.email)
    .bind(&payload.name)
    .bind(&password_hash)
    .bind(role)
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
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    // First check if user exists and belongs to caller's org
    let existing = sqlx::query_as::<_, User>(
        "SELECT id, organisation_id, email, name, role, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("update_user fetch error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let existing = match existing {
        Some(u) => u,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Check access: super admins can update all, others only their org
    if !is_super_admin(&headers) {
        if let Some(caller_org) = get_org_id_from_headers(&headers) {
            if existing.organisation_id != Some(caller_org) {
                return Err(StatusCode::FORBIDDEN);
            }
            // Non-super admins cannot move users to another org
            if let Some(new_org) = payload.organisation_id {
                if new_org != caller_org {
                    return Err(StatusCode::FORBIDDEN);
                }
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Hash password if provided
    let password_hash = if let Some(ref password) = payload.password {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Some(
            argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|err| {
                    eprintln!("update_user password hash error: {err}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .to_string(),
        )
    } else {
        None
    };

    let user = sqlx::query_as::<_, User>(
        r#"UPDATE users SET 
            email = COALESCE($1, email), 
            name = COALESCE($2, name), 
            password_hash = COALESCE($3, password_hash),
            organisation_id = COALESCE($4, organisation_id),
            role = COALESCE($5, role),
            updated_at = NOW()
        WHERE id = $6 
        RETURNING id, organisation_id, email, name, role, created_at, updated_at"#,
    )
    .bind(&payload.email)
    .bind(&payload.name)
    .bind(&password_hash)
    .bind(payload.organisation_id)
    .bind(payload.role)
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
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // First check if user exists and belongs to caller's org
    let existing = sqlx::query_as::<_, User>(
        "SELECT id, organisation_id, email, name, role, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("delete_user fetch error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let existing = match existing {
        Some(u) => u,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Check access: super admins can delete all, others only their org
    if !is_super_admin(&headers) {
        if let Some(caller_org) = get_org_id_from_headers(&headers) {
            if existing.organisation_id != Some(caller_org) {
                return Err(StatusCode::FORBIDDEN);
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let user = sqlx::query_as::<_, User>(
        "DELETE FROM users WHERE id = $1 RETURNING id, organisation_id, email, name, role, created_at, updated_at",
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

use axum::{
    Router, Json,
    extract::{State, Path, Query},
    routing::{get, post, put, delete},
    Extension,
};
use serde::Deserialize;
use uuid::Uuid;
use serde_json::json;

use crate::{
    AppState,
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::user::{CreateUserRequest, UpdateUserRequest, UserResponse, User},
};

#[derive(Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/",            get(list_users).post(create_user))
        .route("/role-counts", get(get_role_counts))
        .route("/:id",         get(get_user).put(update_user).delete(delete_user))
}

async fn list_users(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * per_page;

    let search = q.search.as_deref().unwrap_or("");
    let search_pattern = format!("%{}%", search);

    let users = sqlx::query_as::<_, User>(
        r#"SELECT u.*, COALESCE(d.device_count, 0) as device_count
           FROM users u
           LEFT JOIN (
               SELECT user_id, COUNT(*) as device_count
               FROM user_devices
               GROUP BY user_id
           ) d ON u.id = d.user_id
           WHERE ($1 = '' OR u.name ILIKE $1 OR u.username ILIKE $1 OR u.email ILIKE $1)
             AND ($2 IS NULL OR u.role = $2)
             AND ($3 IS NULL OR u.status = $3)
           ORDER BY u.created_at DESC
           LIMIT $4 OFFSET $5"#
    )
    .bind(&search_pattern)
    .bind(&q.role)
    .bind(&q.status)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE ($1 = '' OR name ILIKE $1 OR username ILIKE $1)"
    )
    .bind(&search_pattern)
    .fetch_one(&state.db.pool)
    .await?;

    let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

    Ok(Json(json!({
        "success": true,
        "data": response,
        "total": total,
        "page": page,
        "per_page": per_page,
        "total_pages": (total as f64 / per_page as f64).ceil() as i64,
    })))
}

async fn get_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" && auth.id != id {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    let user = sqlx::query_as::<_, User>(
        r#"SELECT u.*, COALESCE(d.device_count, 0) as device_count
           FROM users u
           LEFT JOIN (
               SELECT user_id, COUNT(*) as device_count
               FROM user_devices
               GROUP BY user_id
           ) d ON u.id = d.user_id
           WHERE u.id = $1"#
    )
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} tidak ditemukan", id)))?;

    Ok(Json(json!({ "success": true, "data": UserResponse::from(user) })))
}

async fn create_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    // Check username uniqueness
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 OR email = $2)"
    )
    .bind(&req.username)
    .bind(&req.email)
    .fetch_one(&state.db.pool)
    .await?;

    if exists {
        return Err(AppError::Conflict("Username atau email sudah digunakan".into()));
    }

    let hash = bcrypt::hash(&req.password, 12)
        .map_err(|e| AppError::Internal(e.into()))?;

    let user = sqlx::query_as::<_, User>(
        r#"WITH inserted AS (
               INSERT INTO users (id, name, username, email, password_hash, role, status)
               VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6)
               RETURNING *
           )
           SELECT u.*, 0::BIGINT as device_count
           FROM inserted u"#
    )
    .bind(&req.name)
    .bind(&req.username)
    .bind(&req.email)
    .bind(&hash)
    .bind(&req.role)
    .bind(req.status.as_deref().unwrap_or("active"))
    .fetch_one(&state.db.pool)
    .await?;

    if user.role == "investor" {
        let _ = sqlx::query!(
            "INSERT INTO investors (id, user_id, name, email, status) VALUES (gen_random_uuid(), $1, $2, $3, $4)",
            user.id, user.name, user.email, user.status
        )
        .execute(&state.db.pool)
        .await;
    }

    Ok(Json(json!({ "success": true, "data": UserResponse::from(user) })))
}

async fn update_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" && auth.id != id {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    let new_hash = if let Some(ref pw) = req.password {
        Some(bcrypt::hash(pw, 12).map_err(|e| AppError::Internal(e.into()))?)
    } else { None };

    let user = sqlx::query_as::<_, User>(
        r#"WITH updated AS (
               UPDATE users SET
                 name         = COALESCE($1, name),
                 email        = COALESCE($2, email),
                 password_hash= COALESCE($3, password_hash),
                 role         = COALESCE($4, role),
                 status       = COALESCE($5, status),
                 updated_at   = NOW()
               WHERE id = $6 RETURNING *
           )
           SELECT u.*, COALESCE(d.device_count, 0) as device_count
           FROM updated u
           LEFT JOIN (
               SELECT user_id, COUNT(*) as device_count
               FROM user_devices
               GROUP BY user_id
           ) d ON u.id = d.user_id"#
    )
    .bind(&req.name)
    .bind(&req.email)
    .bind(&new_hash)
    .bind(&req.role)
    .bind(&req.status)
    .bind(id)
    .fetch_optional(&state.db.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {} tidak ditemukan", id)))?;

    if user.role == "investor" {
        let updated = sqlx::query!(
            "UPDATE investors SET name = $1, email = $2, status = $3, updated_at = NOW() WHERE user_id = $4",
            user.name, user.email, user.status, user.id
        )
        .execute(&state.db.pool)
        .await?
        .rows_affected();

        if updated == 0 {
            let _ = sqlx::query!(
                "INSERT INTO investors (id, user_id, name, email, status) VALUES (gen_random_uuid(), $1, $2, $3, $4)",
                user.id, user.name, user.email, user.status
            )
            .execute(&state.db.pool)
            .await;
        }
    } else {
        let _ = sqlx::query!("DELETE FROM investors WHERE user_id = $1", user.id)
            .execute(&state.db.pool)
            .await;
    }

    // Invalidate user cache
    let _ = state.cache.del(&format!("user:{}", id)).await;

    Ok(Json(json!({ "success": true, "data": UserResponse::from(user) })))
}

async fn delete_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }
    if auth.id == id {
        return Err(AppError::BadRequest("Tidak dapat menghapus akun sendiri".into()));
    }

    // Check if user has transactions
    let has_tx: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM transactions WHERE cashier_id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db.pool)
    .await?;

    if has_tx {
        return Err(AppError::Conflict(
            "User tidak dapat dihapus karena memiliki riwayat transaksi kasir. Silakan ubah status user menjadi tidak aktif.".into()
        ));
    }

    // Check if user has scanned tickets
    let has_scans: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM ticket_scans WHERE officer_id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db.pool)
    .await?;

    if has_scans {
        return Err(AppError::Conflict(
            "User tidak dapat dihapus karena memiliki riwayat pemindaian tiket. Silakan ubah status user menjadi tidak aktif.".into()
        ));
    }

    // Delete investor record if it exists
    let _ = sqlx::query!("DELETE FROM investors WHERE user_id = $1", id)
        .execute(&state.db.pool)
        .await;

    let rows = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&state.db.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("User {} tidak ditemukan", id)));
    }

    Ok(Json(json!({ "success": true, "data": null })))
}

async fn get_role_counts(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    struct RoleCount {
        role: String,
        count: i64,
    }

    let counts = sqlx::query_as!(
        RoleCount,
        r#"SELECT role, COUNT(*) as "count!" FROM users GROUP BY role"#
    )
    .fetch_all(&state.db.pool)
    .await?;

    let mut map = serde_json::Map::new();
    map.insert("admin".to_string(), json!(0));
    map.insert("kasir".to_string(), json!(0));
    map.insert("officer".to_string(), json!(0));
    map.insert("investor".to_string(), json!(0));

    for item in counts {
        map.insert(item.role, json!(item.count));
    }

    Ok(Json(json!({
        "success": true,
        "data": map
    })))
}


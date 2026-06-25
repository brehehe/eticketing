use axum::{
    Router, Json,
    extract::{State, Path},
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
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/",    get(list).post(create))
        .route("/:id", get(get_one).put(update).delete(del))
}

async fn list(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let rows = sqlx::query!(
        r#"SELECT id, name, slug, icon, sort_order, created_at
           FROM categories ORDER BY sort_order, name"#
    )
    .fetch_all(&state.db.pool)
    .await?;

    let data: Vec<serde_json::Value> = rows.into_iter().map(|r| json!({
        "id": r.id, "name": r.name, "slug": r.slug,
        "icon": r.icon, "sort_order": r.sort_order,
        "created_at": r.created_at,
    })).collect();

    Ok(Json(json!({ "success": true, "data": data })))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let r = sqlx::query!(
        "SELECT id, name, slug, icon, sort_order, created_at FROM categories WHERE id = $1",
        id
    )
    .fetch_optional(&state.db.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Kategori tidak ditemukan".into()))?;

    Ok(Json(json!({ "success": true, "data": {
        "id": r.id, "name": r.name, "slug": r.slug,
        "icon": r.icon, "sort_order": r.sort_order,
    }})))
}

#[derive(Deserialize)]
pub struct UpsertRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
}

async fn create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<UpsertRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" { return Err(AppError::Forbidden("Akses ditolak".into())); }
    let name = req.name.ok_or_else(|| AppError::BadRequest("Nama diperlukan".into()))?;
    let slug = req.slug.unwrap_or_else(|| name.to_lowercase().replace(' ', "-"));

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM categories WHERE slug = $1)")
        .bind(&slug).fetch_one(&state.db.pool).await?;
    if exists { return Err(AppError::Conflict("Slug sudah digunakan".into())); }

    let r = sqlx::query!(
        r#"INSERT INTO categories (id, name, slug, icon, sort_order)
           VALUES (gen_random_uuid(), $1, $2, $3, COALESCE($4, 0))
           RETURNING id, name, slug, icon, sort_order"#,
        name, slug, req.icon, req.sort_order
    )
    .fetch_one(&state.db.pool).await?;

    let _ = state.cache.invalidate_prefix("categories:").await;
    Ok(Json(json!({ "success": true, "data": {
        "id": r.id, "name": r.name, "slug": r.slug,
        "icon": r.icon, "sort_order": r.sort_order,
    }})))
}

async fn update(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpsertRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" { return Err(AppError::Forbidden("Akses ditolak".into())); }

    let r = sqlx::query!(
        r#"UPDATE categories SET
             name       = COALESCE($1, name),
             slug       = COALESCE($2, slug),
             icon       = COALESCE($3, icon),
             sort_order = COALESCE($4, sort_order)
           WHERE id = $5
           RETURNING id, name, slug, icon, sort_order"#,
        req.name, req.slug, req.icon, req.sort_order, id
    )
    .fetch_optional(&state.db.pool).await?
    .ok_or_else(|| AppError::NotFound("Kategori tidak ditemukan".into()))?;

    let _ = state.cache.invalidate_prefix("categories:").await;
    Ok(Json(json!({ "success": true, "data": {
        "id": r.id, "name": r.name, "slug": r.slug,
        "icon": r.icon, "sort_order": r.sort_order,
    }})))
}

async fn del(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" { return Err(AppError::Forbidden("Akses ditolak".into())); }

    // Check if any products use this category
    let in_use: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM products WHERE category_id = $1)"
    ).bind(id).fetch_one(&state.db.pool).await?;

    if in_use {
        return Err(AppError::Conflict(
            "Kategori masih digunakan oleh produk. Pindahkan produk terlebih dahulu.".into()
        ));
    }

    let rows = sqlx::query!("DELETE FROM categories WHERE id = $1", id)
        .execute(&state.db.pool).await?.rows_affected();
    if rows == 0 { return Err(AppError::NotFound("Kategori tidak ditemukan".into())); }

    let _ = state.cache.invalidate_prefix("categories:").await;
    Ok(Json(json!({ "success": true, "data": null })))
}

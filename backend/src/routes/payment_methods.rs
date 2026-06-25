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

async fn list(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    // Cast NUMERIC to float8 in SQL to avoid bigdecimal dependency
    let rows = sqlx::query!(
        r#"SELECT id, name, provider,
             fee_pct::float8  AS "fee_pct!",
             fee_flat,
             auto_settlement, status, created_at
           FROM payment_methods
           ORDER BY created_at"#
    )
    .fetch_all(&state.db.pool)
    .await?;

    let data: Vec<serde_json::Value> = rows.into_iter().map(|r| json!({
        "id":              r.id,
        "name":            r.name,
        "provider":        r.provider,
        "fee_pct":         r.fee_pct,
        "fee_flat":        r.fee_flat,
        "auto_settlement": r.auto_settlement,
        "status":          r.status,
        "created_at":      r.created_at,
    })).collect();

    Ok(Json(json!({ "success": true, "data": data })))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let r = sqlx::query!(
        r#"SELECT id, name, provider,
             fee_pct::float8 AS "fee_pct!",
             fee_flat, auto_settlement, status, created_at
           FROM payment_methods WHERE id = $1"#,
        id
    )
    .fetch_optional(&state.db.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Metode pembayaran tidak ditemukan".into()))?;

    Ok(Json(json!({ "success": true, "data": {
        "id": r.id, "name": r.name, "provider": r.provider,
        "fee_pct": r.fee_pct, "fee_flat": r.fee_flat,
        "auto_settlement": r.auto_settlement, "status": r.status,
    }})))
}

#[derive(Debug, Deserialize)]
pub struct UpsertRequest {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub fee_pct: Option<f64>,
    pub fee_flat: Option<i64>,
    pub auto_settlement: Option<bool>,
    pub status: Option<String>,
}

async fn create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<UpsertRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }
    let name = req.name.ok_or_else(|| AppError::BadRequest("Nama diperlukan".into()))?;

    // Insert with explicit cast to avoid NUMERIC bind mismatch
    let r = sqlx::query!(
        r#"INSERT INTO payment_methods
             (id, name, provider, fee_pct, fee_flat, auto_settlement, status)
           VALUES
             (gen_random_uuid(), $1, $2,
              ($3::float8)::numeric,
              $4, $5, COALESCE($6, 'active'))
           RETURNING id, name, provider,
             fee_pct::float8 AS "fee_pct!",
             fee_flat, auto_settlement, status"#,
        name,
        req.provider,
        req.fee_pct.unwrap_or(0.0),
        req.fee_flat.unwrap_or(0),
        req.auto_settlement.unwrap_or(true),
        req.status
    )
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(json!({ "success": true, "data": {
        "id": r.id, "name": r.name, "provider": r.provider,
        "fee_pct": r.fee_pct, "fee_flat": r.fee_flat,
        "auto_settlement": r.auto_settlement, "status": r.status,
    }})))
}

async fn update(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpsertRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    let r = sqlx::query!(
        r#"UPDATE payment_methods SET
             name            = COALESCE($1, name),
             provider        = COALESCE($2, provider),
             fee_pct         = COALESCE(($3::float8)::numeric, fee_pct),
             fee_flat        = COALESCE($4, fee_flat),
             auto_settlement = COALESCE($5, auto_settlement),
             status          = COALESCE($6, status)
           WHERE id = $7
           RETURNING id, name, provider,
             fee_pct::float8 AS "fee_pct!",
             fee_flat, auto_settlement, status"#,
        req.name,
        req.provider,
        req.fee_pct,
        req.fee_flat,
        req.auto_settlement,
        req.status,
        id
    )
    .fetch_optional(&state.db.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Metode tidak ditemukan".into()))?;

    Ok(Json(json!({ "success": true, "data": {
        "id": r.id, "name": r.name, "provider": r.provider,
        "fee_pct": r.fee_pct, "fee_flat": r.fee_flat,
        "auto_settlement": r.auto_settlement, "status": r.status,
    }})))
}

async fn del(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    let rows = sqlx::query!("DELETE FROM payment_methods WHERE id = $1", id)
        .execute(&state.db.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound("Metode tidak ditemukan".into()));
    }

    Ok(Json(json!({ "success": true, "data": null })))
}

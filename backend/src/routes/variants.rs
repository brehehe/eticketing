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
    models::product::deserialize_double_option,
};

/// Nested under /products/:product_id/variants
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/",    get(list).post(create))
        .route("/:id", get(get_one).put(update).delete(del))
}

async fn list(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = sqlx::query!(
        "SELECT id, product_id, name, sku, price, investor_price, quota, quota_used, status, created_at
         FROM product_variants WHERE product_id = $1 ORDER BY name",
        product_id
    )
    .fetch_all(&state.db.pool).await?;

    let data: Vec<serde_json::Value> = rows.into_iter().map(|r| json!({
        "id": r.id, "product_id": r.product_id, "name": r.name,
        "sku": r.sku, "price": r.price, "investor_price": r.investor_price,
        "quota": r.quota, "quota_used": r.quota_used, "status": r.status,
        "created_at": r.created_at,
    })).collect();

    Ok(Json(json!({ "success": true, "data": data })))
}

async fn get_one(
    State(state): State<AppState>,
    Path((product_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    let r = sqlx::query!(
        "SELECT id, product_id, name, sku, price, investor_price, quota, quota_used, status
         FROM product_variants WHERE product_id = $1 AND id = $2",
        product_id, id
    )
    .fetch_optional(&state.db.pool).await?
    .ok_or_else(|| AppError::NotFound("Varian tidak ditemukan".into()))?;

    Ok(Json(json!({ "success": true, "data": {
        "id": r.id, "product_id": r.product_id,
        "name": r.name, "sku": r.sku, "price": r.price, "investor_price": r.investor_price,
        "quota": r.quota, "quota_used": r.quota_used, "status": r.status,
    }})))
}

#[derive(Deserialize)]
pub struct CreateVariantRequest {
    pub name: String,
    pub sku: String,
    pub price: i64,
    pub investor_price: Option<i64>,
    pub quota: Option<i32>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateVariantRequest {
    pub name: Option<String>,
    pub sku: Option<String>,
    pub price: Option<i64>,
    pub investor_price: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub quota: Option<Option<i32>>,
    pub status: Option<String>,
}

async fn create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(product_id): Path<Uuid>,
    Json(req): Json<CreateVariantRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" { return Err(AppError::Forbidden("Akses ditolak".into())); }

    // Verify product exists
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE id = $1)")
        .bind(product_id).fetch_one(&state.db.pool).await?;
    if !exists { return Err(AppError::NotFound("Produk tidak ditemukan".into())); }

    // Check SKU uniqueness
    let sku_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM product_variants WHERE sku = $1)"
    ).bind(&req.sku).fetch_one(&state.db.pool).await?;
    if sku_exists { return Err(AppError::Conflict("SKU sudah digunakan".into())); }

    let r = sqlx::query!(
        r#"INSERT INTO product_variants
             (id, product_id, name, sku, price, investor_price, quota, status)
           VALUES
             (gen_random_uuid(), $1, $2, $3, $4, COALESCE($5, 0::bigint), $6, COALESCE($7, 'active'))
           RETURNING id, product_id, name, sku, price, investor_price, quota, quota_used, status"#,
        product_id, req.name, req.sku, req.price, req.investor_price, req.quota, req.status
    )
    .fetch_one(&state.db.pool).await?;

    // Enable has_variants on the parent product
    sqlx::query!("UPDATE products SET has_variants = true WHERE id = $1", product_id)
        .execute(&state.db.pool).await?;

    Ok(Json(json!({ "success": true, "data": {
        "id": r.id, "product_id": r.product_id,
        "name": r.name, "sku": r.sku, "price": r.price, "investor_price": r.investor_price,
        "quota": r.quota, "quota_used": r.quota_used, "status": r.status,
    }})))
}

async fn update(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((_product_id, id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateVariantRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" { return Err(AppError::Forbidden("Akses ditolak".into())); }

    let (quota_present, quota_val) = match req.quota {
        Some(val) => (true, val),
        None => (false, None),
    };

    let r = sqlx::query!(
        r#"UPDATE product_variants SET
             name           = COALESCE($1, name),
             sku            = COALESCE($2, sku),
             price          = COALESCE($3, price),
             investor_price = COALESCE($4, investor_price),
             quota          = CASE WHEN $5::boolean THEN $6::integer ELSE quota END,
             status         = COALESCE($7, status)
           WHERE id = $8
           RETURNING id, product_id, name, sku, price, investor_price, quota, quota_used, status"#,
        req.name, req.sku, req.price, req.investor_price, quota_present, quota_val, req.status, id
    )
    .fetch_optional(&state.db.pool).await?
    .ok_or_else(|| AppError::NotFound("Varian tidak ditemukan".into()))?;

    Ok(Json(json!({ "success": true, "data": {
        "id": r.id, "product_id": r.product_id,
        "name": r.name, "sku": r.sku, "price": r.price, "investor_price": r.investor_price,
        "quota": r.quota, "quota_used": r.quota_used, "status": r.status,
    }})))
}

async fn del(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((_product_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" { return Err(AppError::Forbidden("Akses ditolak".into())); }

    // Check if any transactions reference this variant
    let in_use_tx: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM transaction_items WHERE variant_id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db.pool)
    .await?;

    if in_use_tx {
        return Err(AppError::Conflict(
            "Varian tidak dapat dihapus karena memiliki riwayat transaksi. Silakan ubah status varian menjadi tidak aktif.".into()
        ));
    }

    // Check if any tickets reference this variant
    let in_use_tickets: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM tickets WHERE variant_id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db.pool)
    .await?;

    if in_use_tickets {
        return Err(AppError::Conflict(
            "Varian tidak dapat dihapus karena memiliki tiket terkait. Silakan ubah status varian menjadi tidak aktif.".into()
        ));
    }

    let r = sqlx::query!(
        "DELETE FROM product_variants WHERE id = $1 RETURNING product_id", id
    )
    .fetch_optional(&state.db.pool).await?
    .ok_or_else(|| AppError::NotFound("Varian tidak ditemukan".into()))?;

    // If no more variants, set has_variants = false
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM product_variants WHERE product_id = $1"
    ).bind(r.product_id).fetch_one(&state.db.pool).await?;

    if count == 0 {
        sqlx::query!("UPDATE products SET has_variants = false WHERE id = $1", r.product_id)
            .execute(&state.db.pool).await?;
    }

    Ok(Json(json!({ "success": true, "data": null })))
}

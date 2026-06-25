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
    models::product::{CreateProductRequest, UpdateProductRequest, Product},
};

#[derive(Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub status: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/",    get(list_products).post(create_product))
        .route("/:id", get(get_product).put(update_product).delete(delete_product))
}

async fn list_products(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).min(200).max(1);
    let offset = (page - 1) * per_page;
    let search = format!("%{}%", q.search.as_deref().unwrap_or(""));

    let query_investor_id = if auth.role == "investor" {
        let investor_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM investors WHERE user_id = $1"
        )
        .bind(auth.id)
        .fetch_optional(&state.db.pool)
        .await?;

        if investor_id.is_none() {
            return Err(AppError::Forbidden("Anda tidak terdaftar sebagai investor aktif".into()));
        }
        investor_id
    } else {
        None
    };

    let products = sqlx::query_as::<_, Product>(
        r#"SELECT * FROM products
           WHERE (name ILIKE $1 OR sku ILIKE $1)
             AND ($2::text IS NULL OR status = $2)
             AND ($5::uuid IS NULL OR investor_id = $5)
           ORDER BY name ASC
           LIMIT $3 OFFSET $4"#
    )
    .bind(&search)
    .bind(&q.status)
    .bind(per_page)
    .bind(offset)
    .bind(query_investor_id)
    .fetch_all(&state.db.pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*)::bigint FROM products
           WHERE (name ILIKE $1 OR sku ILIKE $1)
             AND ($2::text IS NULL OR status = $2)
             AND ($3::uuid IS NULL OR investor_id = $3)"#
    )
    .bind(&search)
    .bind(&q.status)
    .bind(query_investor_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(json!({
        "success": true,
        "data": products,
        "total": total,
        "page": page,
        "per_page": per_page,
    })))
}

async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Produk {} tidak ditemukan", id)))?;

    let variants = sqlx::query!(
        r#"SELECT
             id, product_id, name, sku, price, investor_price, quota, quota_used, status,
             created_at
           FROM product_variants WHERE product_id = $1 ORDER BY name"#,
        id
    )
    .fetch_all(&state.db.pool)
    .await?;

    let variants_json: Vec<serde_json::Value> = variants.into_iter().map(|v| json!({
        "id": v.id,
        "product_id": v.product_id,
        "name": v.name,
        "sku": v.sku,
        "price": v.price,
        "investor_price": v.investor_price,
        "quota": v.quota,
        "quota_used": v.quota_used,
        "status": v.status,
        "created_at": v.created_at,
    })).collect();

    Ok(Json(json!({ "success": true, "data": { "product": product, "variants": variants_json } })))
}

async fn create_product(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<CreateProductRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE sku = $1)")
        .bind(&req.sku)
        .fetch_one(&state.db.pool)
        .await?;
    if exists {
        return Err(AppError::Conflict("SKU sudah digunakan".into()));
    }

    let product = sqlx::query_as::<_, Product>(
        r#"INSERT INTO products
             (id, name, sku, category_id, price, investor_price, investor_id, quota, status,
              has_variants, ticket_required, print_enabled, scan_enabled)
           VALUES
             (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7,
              COALESCE($8, 'active'),
              COALESCE($9, false), COALESCE($10, true),
              COALESCE($11, true), COALESCE($12, true))
           RETURNING *"#
    )
    .bind(&req.name).bind(&req.sku).bind(req.category_id)
    .bind(req.price).bind(req.investor_price.unwrap_or(0)).bind(req.investor_id).bind(req.quota)
    .bind(&req.status)
    .bind(req.has_variants).bind(req.ticket_required)
    .bind(req.print_enabled).bind(req.scan_enabled)
    .fetch_one(&state.db.pool)
    .await?;

    let _ = state.cache.invalidate_prefix("products:").await;
    Ok(Json(json!({ "success": true, "data": product })))
}

async fn update_product(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    let (investor_id_present, investor_id_val) = match req.investor_id {
        Some(val) => (true, val),
        None => (false, None),
    };

    let (quota_present, quota_val) = match req.quota {
        Some(val) => (true, val),
        None => (false, None),
    };

    let product = sqlx::query_as::<_, Product>(
        r#"UPDATE products SET
             name           = COALESCE($1, name),
             price          = COALESCE($2, price),
             investor_price = COALESCE($3, investor_price),
             investor_id    = CASE WHEN $4::boolean THEN $5::uuid ELSE investor_id END,
             quota          = CASE WHEN $6::boolean THEN $7::integer ELSE quota END,
             status         = COALESCE($8, status),
             ticket_required= COALESCE($9, ticket_required),
             print_enabled  = COALESCE($10, print_enabled),
             scan_enabled   = COALESCE($11, scan_enabled),
             updated_at     = NOW()
           WHERE id = $12 RETURNING *"#
    )
    .bind(&req.name).bind(req.price).bind(req.investor_price)
    .bind(investor_id_present).bind(investor_id_val)
    .bind(quota_present).bind(quota_val)
    .bind(&req.status).bind(req.ticket_required)
    .bind(req.print_enabled).bind(req.scan_enabled)
    .bind(id)
    .fetch_optional(&state.db.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Produk {} tidak ditemukan", id)))?;

    let _ = state.cache.invalidate_prefix("products:").await;
    Ok(Json(json!({ "success": true, "data": product })))
}

async fn delete_product(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    // Check if any transactions reference this product
    let in_use_tx: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM transaction_items WHERE product_id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db.pool)
    .await?;

    if in_use_tx {
        return Err(AppError::Conflict(
            "Produk tidak dapat dihapus karena memiliki riwayat transaksi. Silakan ubah status produk menjadi tidak aktif.".into()
        ));
    }

    // Check if any tickets reference this product
    let in_use_tickets: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM tickets WHERE product_id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db.pool)
    .await?;

    if in_use_tickets {
        return Err(AppError::Conflict(
            "Produk tidak dapat dihapus karena memiliki tiket terkait. Silakan ubah status produk menjadi tidak aktif.".into()
        ));
    }

    let rows = sqlx::query!("DELETE FROM products WHERE id = $1", id)
        .execute(&state.db.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("Produk {} tidak ditemukan", id)));
    }

    let _ = state.cache.invalidate_prefix("products:").await;
    Ok(Json(json!({ "success": true, "data": null })))
}

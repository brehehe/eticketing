use axum::{
    Router, Json,
    extract::{State, Path, Query},
    routing::{get, post},
    Extension,
};
use serde::Deserialize;
use uuid::Uuid;
use serde_json::json;

use crate::{
    AppState,
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::transaction::{CreateTransactionRequest, Transaction},
};

#[derive(Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub status: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/",            get(list_transactions).post(create_transaction))
        .route("/:id",         get(get_transaction))
        .route("/:id/refund",  post(refund_transaction))
        .route("/:id/reprint", post(reprint_transaction))
}

async fn list_transactions(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let transactions = sqlx::query_as::<_, Transaction>(
        r#"SELECT * FROM transactions
           WHERE ($1::text IS NULL OR status = $1)
             AND ($2::timestamptz IS NULL OR created_at >= $2::timestamptz)
             AND ($3::timestamptz IS NULL OR created_at <= $3::timestamptz)
           ORDER BY created_at DESC
           LIMIT $4 OFFSET $5"#
    )
    .bind(&q.status)
    .bind(&q.from)
    .bind(&q.to)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM transactions")
        .fetch_one(&state.db.pool)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": transactions,
        "total": total,
        "page": page,
        "per_page": per_page,
    })))
}

async fn get_transaction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let tx = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Transaksi tidak ditemukan".into()))?;

    let items = sqlx::query!(
        r#"SELECT
             ti.id, ti.product_id, ti.variant_id, ti.qty,
             ti.unit_price, ti.discount, ti.subtotal,
             p.name AS product_name
           FROM transaction_items ti
           JOIN products p ON p.id = ti.product_id
           WHERE ti.transaction_id = $1"#,
        id
    )
    .fetch_all(&state.db.pool)
    .await?;

    let items_json: Vec<serde_json::Value> = items.into_iter().map(|i| json!({
        "id": i.id,
        "product_id": i.product_id,
        "variant_id": i.variant_id,
        "product_name": i.product_name,
        "qty": i.qty,
        "unit_price": i.unit_price,
        "discount": i.discount,
        "subtotal": i.subtotal,
    })).collect();

    let payments = sqlx::query!(
        "SELECT id, method, amount, fee, reference, status, paid_at FROM payments WHERE transaction_id = $1",
        id
    )
    .fetch_all(&state.db.pool)
    .await?;

    let payments_json: Vec<serde_json::Value> = payments.into_iter().map(|p| json!({
        "id": p.id,
        "method": p.method,
        "amount": p.amount,
        "fee": p.fee,
        "reference": p.reference,
        "status": p.status,
        "paid_at": p.paid_at,
    })).collect();

    Ok(Json(json!({
        "success": true,
        "data": { "transaction": tx, "items": items_json, "payments": payments_json }
    })))
}

async fn create_transaction(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<CreateTransactionRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if req.items.is_empty() {
        return Err(AppError::BadRequest("Minimal 1 item diperlukan".into()));
    }
    if req.payments.is_empty() {
        return Err(AppError::BadRequest("Minimal 1 metode pembayaran diperlukan".into()));
    }

    let mut db_tx = state.db.pool.begin().await?;

    // Invoice format: TK-YYYYMMDD-XXXXX (date + daily sequence)
    let today = chrono::Utc::now().format("%Y%m%d").to_string();
    let daily_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM transactions WHERE DATE(created_at) = CURRENT_DATE"
    )
    .fetch_one(&mut *db_tx)
    .await?;
    let invoice = format!("TK-{}-{:05}", today, daily_count + 1);

    let mut subtotal: i64 = 0;
    let mut item_rows: Vec<(Uuid, Option<Uuid>, i32, i64, i64, i64)> = Vec::new();

    for item in &req.items {
        let product = sqlx::query!(
            "SELECT id, price FROM products WHERE id = $1 AND status = 'active'",
            item.product_id
        )
        .fetch_optional(&mut *db_tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Produk {} tidak ditemukan", item.product_id)))?;

        let unit_price = if let Some(vid) = item.variant_id {
            let v = sqlx::query!("SELECT price FROM product_variants WHERE id = $1", vid)
                .fetch_optional(&mut *db_tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Varian tidak ditemukan".into()))?;
            v.price
        } else {
            product.price
        };

        let item_discount = item.discount.unwrap_or(0);
        let item_subtotal = unit_price * item.qty as i64 - item_discount;
        subtotal += item_subtotal;
        item_rows.push((item.product_id, item.variant_id, item.qty, unit_price, item_discount, item_subtotal));
    }

    let discount = req.discount.unwrap_or(0);
    let total = (subtotal - discount).max(0);
    let paid: i64 = req.payments.iter().map(|p| p.amount).sum();
    let change = (paid - total).max(0);

    let tx_row = sqlx::query_as::<_, Transaction>(
        r#"INSERT INTO transactions
             (id, invoice, cashier_id, status, subtotal, discount, total, change, paid_at)
           VALUES
             (gen_random_uuid(), $1, $2, 'paid', $3, $4, $5, $6, NOW())
           RETURNING *"#
    )
    .bind(&invoice).bind(auth.id)
    .bind(subtotal).bind(discount).bind(total).bind(change)
    .fetch_one(&mut *db_tx)
    .await?;

    for (product_id, variant_id, qty, unit_price, item_discount, item_subtotal) in &item_rows {
        sqlx::query(
            "INSERT INTO transaction_items \
             (id, transaction_id, product_id, variant_id, qty, unit_price, discount, subtotal) \
             VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(tx_row.id)
        .bind(product_id)
        .bind(variant_id)
        .bind(qty)
        .bind(unit_price)
        .bind(item_discount)
        .bind(item_subtotal)
        .execute(&mut *db_tx)
        .await?;

        if let Some(vid) = variant_id {
            sqlx::query(
                "UPDATE product_variants SET quota_used = quota_used + $1 WHERE id = $2"
            )
            .bind(qty)
            .bind(vid)
            .execute(&mut *db_tx).await?;
        } else {
            sqlx::query(
                "UPDATE products SET quota_used = quota_used + $1 WHERE id = $2"
            )
            .bind(qty)
            .bind(product_id)
            .execute(&mut *db_tx).await?;
        }
    }

    for payment in &req.payments {
        sqlx::query!(
            r#"INSERT INTO payments
                 (id, transaction_id, method, amount, fee, status, paid_at)
               VALUES (gen_random_uuid(), $1, $2, $3, 0, 'paid', NOW())"#,
            tx_row.id, payment.method, payment.amount
        )
        .execute(&mut *db_tx)
        .await?;
    }

    // Generate tickets for ticket_required products
    for (i, (product_id, variant_id, qty, ..)) in item_rows.iter().enumerate() {
        let needs_ticket: bool = sqlx::query_scalar(
            "SELECT ticket_required FROM products WHERE id = $1"
        )
        .bind(product_id)
        .fetch_one(&mut *db_tx)
        .await?;

        if needs_ticket {
            // Get ticket expiry from settings (default 0 = no expiry)
            let expiry_hours: i64 = sqlx::query_scalar(
                "SELECT COALESCE((value::jsonb->>'expiry_hours')::bigint, 0) FROM settings WHERE key = 'ticket'"
            )
            .fetch_optional(&mut *db_tx)
            .await?
            .flatten()
            .unwrap_or(0);

            let valid_until = if expiry_hours > 0 {
                Some(chrono::Utc::now() + chrono::Duration::hours(expiry_hours))
            } else {
                None
            };

            // Get ticket prefix from settings
            let prefix: String = sqlx::query_scalar(
                "SELECT COALESCE(value::jsonb->>'ticket_prefix', 'TK') FROM settings WHERE key = 'ticket'"
            )
            .fetch_optional(&mut *db_tx)
            .await?
            .flatten()
            .unwrap_or_else(|| "TK".to_string());

            // Ticket sequence: count existing tickets today for this transaction
            let ticket_seq_base: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)::bigint FROM tickets WHERE DATE(created_at) = CURRENT_DATE"
            )
            .fetch_one(&mut *db_tx)
            .await?;

            let product_daily_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)::bigint FROM tickets WHERE product_id = $1 AND DATE(created_at) = CURRENT_DATE"
            )
            .bind(product_id)
            .fetch_one(&mut *db_tx)
            .await?;

            for j in 0..*qty {
                let queue_num = (product_daily_count + j as i64 + 1) as i32;
                let ticket_num = ticket_seq_base + (i as i64 * 100) + j as i64 + 1;
                let date_part = chrono::Utc::now().format("%Y%m%d").to_string();
                let code = format!("{}-{}-{:04}", prefix, date_part, ticket_num);
                let qr_data = format!("TIKETKU:{}:{}", tx_row.id, code);
                sqlx::query(
                    "INSERT INTO tickets \
                     (id, code, qr_data, transaction_id, product_id, variant_id, queue_number, status, valid_until) \
                     VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, 'active', $7)"
                )
                .bind(&code)
                .bind(&qr_data)
                .bind(tx_row.id)
                .bind(product_id)
                .bind(variant_id)
                .bind(queue_num)
                .bind(valid_until)
                .execute(&mut *db_tx)
                .await?;
            }
        }
    }

    db_tx.commit().await?;
    let _ = state.cache.invalidate_prefix("dashboard:").await;

    Ok(Json(json!({ "success": true, "data": { "transaction": tx_row, "invoice": invoice } })))
}

async fn refund_transaction(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let tx = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Transaksi tidak ditemukan".into()))?;

    if tx.status != "paid" {
        return Err(AppError::BadRequest("Hanya transaksi paid yang bisa direfund".into()));
    }

    sqlx::query!("UPDATE transactions SET status = 'refunded', updated_at = NOW() WHERE id = $1", id)
        .execute(&state.db.pool).await?;
    sqlx::query!("UPDATE tickets SET status = 'refunded' WHERE transaction_id = $1", id)
        .execute(&state.db.pool).await?;

    let _ = state.cache.invalidate_prefix("dashboard:").await;
    Ok(Json(json!({ "success": true, "data": null })))
}

async fn reprint_transaction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Return tickets with product name, variant name, valid_until for print
    let tickets = sqlx::query!(
        r#"SELECT
             t.id, t.code, t.qr_data, t.valid_until, t.status, t.queue_number,
             p.name AS "product_name!",
             pv.name AS "variant_name?"
           FROM tickets t
           JOIN products p ON p.id = t.product_id
           LEFT JOIN product_variants pv ON pv.id = t.variant_id
           WHERE t.transaction_id = $1
           ORDER BY t.created_at"#,
        id
    )
    .fetch_all(&state.db.pool)
    .await?;

    let data: Vec<serde_json::Value> = tickets.into_iter().map(|t| json!({
        "id":           t.id,
        "code":         t.code,
        "qr_data":      t.qr_data,
        "product_name": t.product_name,
        "variant_name": t.variant_name,
        "valid_until":  t.valid_until,
        "status":       t.status,
        "queue_number": t.queue_number,
    })).collect();

    Ok(Json(json!({ "success": true, "data": data })))
}

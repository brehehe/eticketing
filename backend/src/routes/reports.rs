use axum::{
    Router, Json,
    extract::{State, Query},
    routing::get,
    Extension,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
};

#[derive(Deserialize)]
pub struct ReportQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub cashier_id: Option<String>,
    pub investor_id: Option<String>,
    pub product_id: Option<String>,
}

#[derive(Serialize)]
struct SalesRow {
    date: Option<String>,
    total_transactions: Option<i64>,
    total_revenue: Option<i64>,
    total_refunds: Option<i64>,
    net_revenue: Option<i64>,
}

#[derive(Serialize)]
struct ProductRow {
    product_id: Option<String>,
    product_name: Option<String>,
    category: Option<String>,
    qty_sold: Option<i64>,
    revenue: Option<i64>,
    investor_price: Option<i64>,
    total_hpp: Option<i64>,
    net_profit: Option<i64>,
}

#[derive(Serialize)]
struct HourlyRow {
    hour: String,
    revenue: i64,
}

#[derive(Serialize)]
struct WeeklyRow {
    date: String,
    revenue: i64,
}

#[derive(Serialize)]
struct InvestorRow {
    investor_id: String,
    investor_name: String,
    email: Option<String>,
    total_products: i64,
    qty_sold: i64,
    total_revenue: i64,
    investor_revenue: i64,
    system_revenue: i64,
}

#[derive(Serialize)]
struct PaymentMethodRow {
    method: String,
    total_transactions: i64,
    total_revenue: i64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sales",     get(sales_report))
        .route("/products",  get(product_report))
        .route("/investors", get(investor_report))
        .route("/payment-methods", get(payment_method_report))
        .route("/dashboard", get(dashboard_stats))
}

async fn sales_report(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ReportQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let from = q.from.clone().unwrap_or_else(|| "2000-01-01".to_string());
    let to   = q.to.clone().unwrap_or_else(|| "2099-01-01".to_string());

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
        q.investor_id.as_ref().and_then(|id| Uuid::parse_str(id).ok())
    };

    let query_product_id = q.product_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());

    let rows = sqlx::query!(
        r#"SELECT
             DATE(t.created_at)::text                                                     AS "date",
             COUNT(DISTINCT t.id)::bigint                                                 AS "total_transactions!",
             COALESCE(SUM(ti.subtotal)::bigint, 0)                                        AS "total_revenue!",
             COALESCE(SUM(CASE WHEN t.status='refunded' THEN ti.subtotal ELSE 0 END)::bigint, 0) AS "total_refunds!",
             COALESCE(SUM(CASE WHEN t.status='paid'     THEN ti.subtotal ELSE 0 END)::bigint, 0) AS "net_revenue!"
           FROM transactions t
           JOIN transaction_items ti ON ti.transaction_id = t.id
           JOIN products p ON p.id = ti.product_id
           WHERE t.created_at BETWEEN $1::date AND $2::date
             AND ($3::uuid IS NULL OR p.investor_id = $3)
             AND ($4::uuid IS NULL OR p.id = $4)
           GROUP BY DATE(t.created_at)
           ORDER BY DATE(t.created_at)"#,
        from as _, to as _, query_investor_id, query_product_id
    )
    .fetch_all(&state.db.pool)
    .await?;

    let data: Vec<SalesRow> = rows.into_iter().map(|r| SalesRow {
        date: r.date,
        total_transactions: Some(r.total_transactions),
        total_revenue: Some(r.total_revenue),
        total_refunds: Some(r.total_refunds),
        net_revenue: Some(r.net_revenue),
    }).collect();

    Ok(Json(json!({ "success": true, "data": data })))
}

async fn product_report(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ReportQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let from = q.from.clone().unwrap_or_else(|| "2000-01-01".to_string());
    let to   = q.to.clone().unwrap_or_else(|| "2099-01-01".to_string());

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
        q.investor_id.as_ref().and_then(|id| Uuid::parse_str(id).ok())
    };

    let query_product_id = q.product_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());

    let rows = sqlx::query!(
        r#"SELECT
             p.id::text                                           AS "product_id!",
             p.name                                               AS "product_name!",
             COALESCE(c.name, 'Uncategorized')                    AS "category!",
             COALESCE(SUM(ti.qty)::bigint, 0)                     AS "qty_sold!",
             COALESCE(SUM(ti.subtotal)::bigint, 0)                AS "revenue!",
             p.investor_price                                     AS "investor_price!",
             COALESCE(SUM(ti.qty * COALESCE(pv.investor_price, p.investor_price))::bigint, 0) AS "total_hpp!",
             COALESCE(SUM(ti.subtotal - (ti.qty * COALESCE(pv.investor_price, p.investor_price)))::bigint, 0) AS "net_profit!"
           FROM transaction_items ti
           JOIN products p ON p.id = ti.product_id
           LEFT JOIN product_variants pv ON pv.id = ti.variant_id
           LEFT JOIN categories c ON c.id = p.category_id
           JOIN transactions t ON t.id = ti.transaction_id
           WHERE t.status = 'paid'
             AND t.created_at BETWEEN $1::date AND $2::date
             AND ($3::uuid IS NULL OR p.investor_id = $3)
             AND ($4::uuid IS NULL OR p.id = $4)
           GROUP BY p.id, p.name, c.name, p.investor_price
           ORDER BY SUM(ti.subtotal) DESC NULLS LAST
           LIMIT 50"#,
        from as _, to as _, query_investor_id, query_product_id
    )
    .fetch_all(&state.db.pool)
    .await?;

    let data: Vec<ProductRow> = rows.into_iter().map(|r| ProductRow {
        product_id: Some(r.product_id),
        product_name: Some(r.product_name),
        category: Some(r.category),
        qty_sold: Some(r.qty_sold),
        revenue: Some(r.revenue),
        investor_price: Some(r.investor_price),
        total_hpp: Some(r.total_hpp),
        net_profit: Some(r.net_profit),
    }).collect();

    Ok(Json(json!({ "success": true, "data": data })))
}

#[derive(Serialize)]
struct InvestorProductRow {
    product_id: String,
    product_name: String,
    investor_id: String,
    investor_name: String,
    qty_sold: i64,
    revenue: i64,
    revenue_pct: i64,
    investor_share: i64,
}

async fn investor_report(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ReportQuery>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" && auth.role != "investor" {
        return Err(AppError::Forbidden("Anda tidak memiliki akses ke laporan investor".into()));
    }

    let from = q.from.clone().unwrap_or_else(|| "2000-01-01".to_string());
    let to   = q.to.clone().unwrap_or_else(|| "2099-01-01".to_string());

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
        q.investor_id.as_ref().and_then(|id| Uuid::parse_str(id).ok())
    };

    let query_product_id = q.product_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());

    let rows = sqlx::query!(
        r#"SELECT
             p.id::text                                           AS "product_id!",
             p.name                                               AS "product_name!",
             i.id::text                                           AS "investor_id!",
             i.name                                               AS "investor_name!",
             COALESCE(SUM(ti.qty)::bigint, 0)                     AS "qty_sold!",
             COALESCE(SUM(ti.subtotal)::bigint, 0)                AS "revenue!",
             COALESCE(SUM(ti.qty * COALESCE(pv.investor_price, p.investor_price))::bigint, 0) AS "investor_share!"
           FROM transaction_items ti
           JOIN products p ON p.id = ti.product_id
           JOIN investors i ON i.id = p.investor_id
           LEFT JOIN product_variants pv ON pv.id = ti.variant_id
           JOIN transactions t ON t.id = ti.transaction_id
           WHERE t.status = 'paid'
             AND t.created_at BETWEEN $1::date AND $2::date
             AND ($3::uuid IS NULL OR i.id = $3)
             AND ($4::uuid IS NULL OR p.id = $4)
           GROUP BY p.id, p.name, i.id, i.name
           HAVING COALESCE(SUM(ti.qty), 0) > 0
           ORDER BY p.name ASC"#,
        from as _, to as _, query_investor_id, query_product_id
    )
    .fetch_all(&state.db.pool)
    .await?;

    let data: Vec<InvestorProductRow> = rows.into_iter().map(|r| {
        let revenue_pct = if r.revenue > 0 {
            (r.investor_share * 100) / r.revenue
        } else {
            0
        };
        InvestorProductRow {
            product_id: r.product_id,
            product_name: r.product_name,
            investor_id: r.investor_id,
            investor_name: r.investor_name,
            qty_sold: r.qty_sold,
            revenue: r.revenue,
            revenue_pct,
            investor_share: r.investor_share,
        }
    }).collect();

    Ok(Json(json!({ "success": true, "data": data })))
}

async fn dashboard_stats(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ReportQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let investor_id: Option<Uuid> = if auth.role == "investor" {
        sqlx::query_scalar("SELECT id FROM investors WHERE user_id = $1")
            .bind(auth.id)
            .fetch_optional(&state.db.pool)
            .await?
    } else {
        None
    };

    let from = q.from.clone().unwrap_or_else(|| "today".to_string());
    let to = q.to.clone().unwrap_or_else(|| "today".to_string());

    let cache_key = if let Some(iid) = investor_id {
        format!("dashboard:stats:investor:{}:{}:{}", iid, from, to)
    } else {
        format!("dashboard:stats:admin:{}:{}", from, to)
    };

    if let Some(cached) = state.cache.get::<serde_json::Value>(&cache_key).await {
        return Ok(Json(cached));
    }

    let sql_from: Option<NaiveDate> = if from == "today" { None } else { NaiveDate::parse_from_str(&from, "%Y-%m-%d").ok() };
    let sql_to: Option<NaiveDate> = if to == "today" { None } else { NaiveDate::parse_from_str(&to, "%Y-%m-%d").ok() };

    let today_revenue: Option<i64> = sqlx::query_scalar(
        "SELECT COALESCE(SUM(ti.subtotal), 0)::bigint \
         FROM transaction_items ti \
         JOIN transactions t ON t.id = ti.transaction_id \
         JOIN products p ON p.id = ti.product_id \
         WHERE t.status = 'paid' \
           AND ($1::uuid IS NULL OR p.investor_id = $1) \
           AND ($2::date IS NULL AND DATE(t.created_at) = CURRENT_DATE OR $2::date IS NOT NULL AND DATE(t.created_at) >= $2::date) \
           AND ($3::date IS NULL AND DATE(t.created_at) = CURRENT_DATE OR $3::date IS NOT NULL AND DATE(t.created_at) <= $3::date)"
    )
    .bind(investor_id)
    .bind(sql_from.as_ref())
    .bind(sql_to.as_ref())
    .fetch_one(&state.db.pool)
    .await?;

    let tickets_today: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint \
         FROM tickets tk \
         JOIN products p ON p.id = tk.product_id \
         WHERE ($1::uuid IS NULL OR p.investor_id = $1) \
           AND ($2::date IS NULL AND DATE(tk.created_at) = CURRENT_DATE OR $2::date IS NOT NULL AND DATE(tk.created_at) >= $2::date) \
           AND ($3::date IS NULL AND DATE(tk.created_at) = CURRENT_DATE OR $3::date IS NOT NULL AND DATE(tk.created_at) <= $3::date)"
    )
    .bind(investor_id)
    .bind(sql_from.as_ref())
    .bind(sql_to.as_ref())
    .fetch_one(&state.db.pool)
    .await?;

    let tx_today: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT t.id)::bigint \
         FROM transactions t \
         JOIN transaction_items ti ON ti.transaction_id = t.id \
         JOIN products p ON p.id = ti.product_id \
         WHERE t.status = 'paid' \
           AND ($1::uuid IS NULL OR p.investor_id = $1) \
           AND ($2::date IS NULL AND DATE(t.created_at) = CURRENT_DATE OR $2::date IS NOT NULL AND DATE(t.created_at) >= $2::date) \
           AND ($3::date IS NULL AND DATE(t.created_at) = CURRENT_DATE OR $3::date IS NOT NULL AND DATE(t.created_at) <= $3::date)"
    )
    .bind(investor_id)
    .bind(sql_from.as_ref())
    .bind(sql_to.as_ref())
    .fetch_one(&state.db.pool)
    .await?;

    let is_multiday = if let (Some(f), Some(t)) = (&sql_from, &sql_to) {
        f != t
    } else {
        false
    };

    let sales_chart: Vec<serde_json::Value> = if is_multiday {
        let daily = sqlx::query!(
            r#"SELECT
                 TO_CHAR(d, 'DD/MM')    AS "date!",
                 COALESCE(SUM(ti.subtotal)::bigint, 0) AS "revenue!"
               FROM GENERATE_SERIES($2::date, $3::date, '1 day'::interval) d
               LEFT JOIN transactions t ON DATE(t.created_at) = DATE(d) AND t.status = 'paid'
               LEFT JOIN transaction_items ti ON ti.transaction_id = t.id
               LEFT JOIN products p ON p.id = ti.product_id AND ($1::uuid IS NULL OR p.investor_id = $1)
               GROUP BY d
               ORDER BY d"#,
            investor_id,
            sql_from,
            sql_to
        )
        .fetch_all(&state.db.pool)
        .await?;

        daily.into_iter().map(|r| json!({
            "date": r.date,
            "revenue": r.revenue,
        })).collect()
    } else {
        let hourly = sqlx::query!(
            r#"SELECT
                 EXTRACT(HOUR FROM t.created_at)::int  AS "hour!",
                 COALESCE(SUM(ti.subtotal)::bigint, 0)     AS "revenue!"
               FROM transactions t
               JOIN transaction_items ti ON ti.transaction_id = t.id
               JOIN products p ON p.id = ti.product_id
               WHERE t.status = 'paid'
                 AND ($1::uuid IS NULL OR p.investor_id = $1)
                 AND ($2::date IS NULL AND DATE(t.created_at) = CURRENT_DATE OR $2::date IS NOT NULL AND DATE(t.created_at) = $2::date)
               GROUP BY EXTRACT(HOUR FROM t.created_at)
               ORDER BY EXTRACT(HOUR FROM t.created_at)"#,
            investor_id,
            sql_from
        )
        .fetch_all(&state.db.pool)
        .await?;

        hourly.into_iter().map(|r| json!({
            "hour": format!("{:02}", r.hour),
            "revenue": r.revenue,
        })).collect()
    };

    let weekly = sqlx::query!(
        r#"SELECT
             TO_CHAR(d, 'Dy')                  AS "date!",
             COALESCE(SUM(ti.subtotal)::bigint, 0) AS "revenue!"
           FROM GENERATE_SERIES(COALESCE($2::date, CURRENT_DATE) - INTERVAL '6 days', COALESCE($2::date, CURRENT_DATE), '1 day'::interval) d
           LEFT JOIN transactions t ON DATE(t.created_at) = DATE(d) AND t.status = 'paid'
           LEFT JOIN transaction_items ti ON ti.transaction_id = t.id
           LEFT JOIN products p ON p.id = ti.product_id AND ($1::uuid IS NULL OR p.investor_id = $1)
           GROUP BY d
           ORDER BY d"#,
        investor_id,
        sql_to
    )
    .fetch_all(&state.db.pool)
    .await?;

    let weekly_data: Vec<WeeklyRow> = weekly.into_iter().map(|r| WeeklyRow {
        date: r.date,
        revenue: r.revenue,
    }).collect();

    let top_product: Option<String> = sqlx::query_scalar(
        r#"SELECT p.name FROM transaction_items ti
           JOIN products p ON p.id = ti.product_id
           JOIN transactions t ON t.id = ti.transaction_id
           WHERE t.status = 'paid'
             AND ($1::uuid IS NULL OR p.investor_id = $1)
             AND ($2::date IS NULL AND DATE(t.created_at) = CURRENT_DATE OR $2::date IS NOT NULL AND DATE(t.created_at) >= $2::date)
             AND ($3::date IS NULL AND DATE(t.created_at) = CURRENT_DATE OR $3::date IS NOT NULL AND DATE(t.created_at) <= $3::date)
           GROUP BY p.id, p.name
           ORDER BY SUM(ti.qty) DESC
           LIMIT 1"#
    )
    .bind(investor_id)
    .bind(sql_from.as_ref())
    .bind(sql_to.as_ref())
    .fetch_optional(&state.db.pool)
    .await?;

    let investor_revenue: Option<i64> = sqlx::query_scalar(
        r#"SELECT COALESCE(SUM(ti.qty * COALESCE(pv.investor_price, pr.investor_price)), 0)::bigint
           FROM transaction_items ti
           JOIN transactions t ON t.id = ti.transaction_id
           JOIN products pr ON pr.id = ti.product_id
           LEFT JOIN product_variants pv ON pv.id = ti.variant_id
           WHERE t.status = 'paid'
             AND ($1::uuid IS NULL OR pr.investor_id = $1)
             AND ($2::date IS NULL AND DATE(t.created_at) = CURRENT_DATE OR $2::date IS NOT NULL AND DATE(t.created_at) >= $2::date)
             AND ($3::date IS NULL AND DATE(t.created_at) = CURRENT_DATE OR $3::date IS NOT NULL AND DATE(t.created_at) <= $3::date)"#
    )
    .bind(investor_id)
    .bind(sql_from.as_ref())
    .bind(sql_to.as_ref())
    .fetch_one(&state.db.pool)
    .await?;

    let stats = json!({
        "success": true,
        "data": {
            "revenue_today":  today_revenue.unwrap_or(0),
            "tickets_sold":   tickets_today.unwrap_or(0),
            "sales_today":    today_revenue.unwrap_or(0),
            "visitors_today": tx_today.unwrap_or(0),
            "top_product":    top_product.unwrap_or_else(|| "-".to_string()),
            "sales_chart":    sales_chart,
            "revenue_chart":  weekly_data,
            "investor_revenue": investor_revenue.unwrap_or(0),
        }
    });

    let _ = state.cache.set(&cache_key, &stats, 120).await;
    Ok(Json(stats))
}

async fn payment_method_report(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ReportQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let from = q.from.clone().unwrap_or_else(|| "2000-01-01".to_string());
    let to   = q.to.clone().unwrap_or_else(|| "2099-01-01".to_string());

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
        q.investor_id.as_ref().and_then(|id| Uuid::parse_str(id).ok())
    };

    let query_product_id = q.product_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());

    let rows = sqlx::query!(
        r#"SELECT
             p.method                                             AS "method!",
             COUNT(DISTINCT p.id)::bigint                         AS "total_transactions!",
             COALESCE(SUM(p.amount)::bigint, 0)                   AS "total_revenue!"
           FROM payments p
           JOIN transactions t ON t.id = p.transaction_id
           WHERE p.status = 'paid'
             AND p.created_at BETWEEN $1::date AND $2::date
             AND ($3::uuid IS NULL OR EXISTS (
                 SELECT 1 FROM transaction_items ti
                 JOIN products pr ON pr.id = ti.product_id
                 WHERE ti.transaction_id = t.id AND pr.investor_id = $3
             ))
             AND ($4::uuid IS NULL OR EXISTS (
                 SELECT 1 FROM transaction_items ti
                 WHERE ti.transaction_id = t.id AND ti.product_id = $4
             ))
           GROUP BY p.method
           ORDER BY SUM(p.amount) DESC"#,
        from as _, to as _, query_investor_id, query_product_id
    )
    .fetch_all(&state.db.pool)
    .await?;

    let data: Vec<PaymentMethodRow> = rows.into_iter().map(|r| PaymentMethodRow {
        method: r.method,
        total_transactions: r.total_transactions,
        total_revenue: r.total_revenue,
    }).collect();

    Ok(Json(json!({ "success": true, "data": data })))
}


use axum::{
    Router, Json,
    extract::{State, Path},
    routing::{get, post},
    Extension,
};
use uuid::Uuid;
use serde_json::json;
use chrono::Utc;

use crate::{
    AppState,
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::ticket::{ScanTicketRequest, ScanResult, Ticket},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/",        get(list_tickets))
        .route("/:id",     get(get_ticket))
        .route("/scan",    post(scan_ticket))
        .route("/validate",post(validate_ticket))
}

async fn list_tickets(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let tickets = sqlx::query_as::<_, Ticket>(
        "SELECT * FROM tickets ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(json!({ "success": true, "data": tickets })))
}

async fn get_ticket(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let ticket = sqlx::query_as::<_, Ticket>("SELECT * FROM tickets WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Tiket tidak ditemukan".into()))?;

    Ok(Json(json!({ "success": true, "data": ticket })))
}

async fn scan_ticket(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<ScanTicketRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let code = req.code.trim().to_string();

    let ticket = sqlx::query_as::<_, Ticket>(
        "SELECT * FROM tickets WHERE code = $1 OR qr_data = $1"
    )
    .bind(&code)
    .fetch_optional(&state.db.pool)
    .await?;

    let (result, message, ticket_data) = match ticket {
        None => (
            "invalid",
            "Tiket tidak ditemukan dalam sistem",
            None,
        ),
        Some(t) if t.status == "used" => (
            "already_used",
            "Tiket sudah pernah digunakan",
            Some(t),
        ),
        Some(t) if t.status == "expired" => (
            "expired",
            "Tiket telah kadaluarsa",
            Some(t),
        ),
        Some(t) if t.status == "refunded" => (
            "invalid",
            "Tiket telah direfund",
            Some(t),
        ),
        Some(t) => {
            // Check expiry
            if let Some(valid_until) = t.valid_until {
                if Utc::now() > valid_until {
                    sqlx::query!("UPDATE tickets SET status = 'expired' WHERE id = $1", t.id)
                        .execute(&state.db.pool)
                        .await?;
                    return Ok(Json(json!({
                        "success": true,
                        "data": { "result": "expired", "message": "Tiket telah kadaluarsa", "ticket": t }
                    })));
                }
            }

            // Mark as used
            sqlx::query!(
                "UPDATE tickets SET status = 'used', used_at = NOW(), used_by = $1 WHERE id = $2",
                auth.id, t.id
            )
            .execute(&state.db.pool)
            .await?;

            // Log scan
            sqlx::query!(
                r#"INSERT INTO ticket_scans
                     (id, ticket_id, officer_id, scan_method, result, scanned_at)
                   VALUES
                     (gen_random_uuid(), $1, $2, $3, 'valid', NOW())"#,
                t.id, auth.id, req.scan_method
            )
            .execute(&state.db.pool)
            .await?;

            ("valid", "Tiket valid, selamat datang!", Some(t))
        }
    };

    // Log failed scans too
    if result != "valid" {
        if let Some(ref t) = ticket_data {
            let _ = sqlx::query!(
                r#"INSERT INTO ticket_scans
                     (id, ticket_id, officer_id, scan_method, result, scanned_at)
                   VALUES
                     (gen_random_uuid(), $1, $2, $3, $4, NOW())"#,
                t.id, auth.id, req.scan_method, result
            )
            .execute(&state.db.pool)
            .await;
        }
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "result": result,
            "message": message,
            "ticket": ticket_data,
        }
    })))
}

async fn validate_ticket(
    State(state): State<AppState>,
    Json(req): Json<ScanTicketRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Dry-run validation (no state change) for preview
    let ticket = sqlx::query_as::<_, Ticket>(
        "SELECT * FROM tickets WHERE code = $1 OR qr_data = $1"
    )
    .bind(&req.code)
    .fetch_optional(&state.db.pool)
    .await?;

    let result = match &ticket {
        None => "invalid",
        Some(t) if t.status == "used" => "already_used",
        Some(t) if t.status == "expired" || t.status == "refunded" => t.status.as_str(),
        Some(_) => "valid",
    };

    Ok(Json(json!({
        "success": true,
        "data": { "result": result, "ticket": ticket }
    })))
}

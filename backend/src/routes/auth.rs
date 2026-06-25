use axum::{
    Router, Json,
    extract::State,
    routing::post,
};
use bcrypt::{hash, verify};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;
use serde_json::json;

use crate::{
    AppState,
    error::{AppError, AppResult},
    models::auth::{LoginRequest, Claims},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login",  post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password",  post(reset_password))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Find user by username or email
    let user = sqlx::query!(
        r#"SELECT id, name, username, email, password_hash, role, status, totp_enabled
           FROM users
           WHERE (username = $1 OR email = $1) AND status = 'active'
           LIMIT 1"#,
        req.username
    )
    .fetch_optional(&state.db.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Username atau password salah".into()))?;

    // Verify password
    let valid = verify(&req.password, &user.password_hash)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("bcrypt error")))?;
    if !valid {
        return Err(AppError::Unauthorized("Username atau password salah".into()));
    }

    // Update last_login
    sqlx::query!("UPDATE users SET last_login = NOW() WHERE id = $1", user.id)
        .execute(&state.db.pool)
        .await?;

    // Build JWT
    let expiry = if req.remember.unwrap_or(false) { 30 * 24 } else { state.config.jwt_expiry_hours as i64 };
    let now = Utc::now();
    let exp = (now + Duration::hours(expiry)).timestamp();
    let jti = Uuid::new_v4().to_string();

    let claims = Claims {
        sub: user.id,
        role: user.role.clone(),
        iat: now.timestamp(),
        exp,
        jti,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    ).map_err(|e| AppError::Internal(e.into()))?;

    let expires_at = (now + Duration::hours(expiry)).to_rfc3339();

    Ok(Json(json!({
        "success": true,
        "data": {
            "token": token,
            "expires_at": expires_at,
            "device_id": req.device_id,
            "user": {
                "id": user.id,
                "name": user.name,
                "username": user.username,
                "email": user.email,
                "role": user.role,
                "status": user.status,
            }
        }
    })))
}

async fn logout(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    // Token revocation is handled by the client removing the token
    // and optionally via Redis blacklist set in middleware
    Ok(Json(json!({ "success": true, "data": null })))
}

async fn refresh(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: implement token refresh
    Ok(Json(json!({ "success": true, "data": null })))
}

async fn forgot_password(
    State(_state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    let _email = req.get("email").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Email diperlukan".into()))?;
    // TODO: send reset email
    Ok(Json(json!({ "success": true, "data": null, "message": "Email instruksi reset dikirim" })))
}

async fn reset_password(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: implement reset
    Ok(Json(json!({ "success": true, "data": null })))
}

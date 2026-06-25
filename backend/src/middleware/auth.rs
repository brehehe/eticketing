use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use uuid::Uuid;

use crate::{AppState, error::AppError, models::auth::Claims};

/// Extension injected into request after successful auth
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: String,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Token tidak ditemukan".into()))?;

    let key = DecodingKey::from_secret(state.config.jwt_secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let data = decode::<Claims>(token, &key, &validation)
        .map_err(|_| AppError::Unauthorized("Token tidak valid atau kadaluarsa".into()))?;

    // Check token revocation via Redis
    let revoked_key = format!("revoked_token:{}", data.claims.jti);
    if state.cache.get::<bool>(&revoked_key).await.is_some() {
        return Err(AppError::Unauthorized("Token telah dicabut".into()));
    }

    req.extensions_mut().insert(AuthUser {
        id: data.claims.sub,
        role: data.claims.role,
    });

    Ok(next.run(req).await)
}

pub async fn require_role(
    role: &'static str,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Unauthorized("Tidak terautentikasi".into()))?;

    if user.role != role && user.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    Ok(next.run(req).await)
}

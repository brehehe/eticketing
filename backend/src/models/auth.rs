use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,       // user_id
    pub role: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,     // unique token id for revocation
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember: Option<bool>,
    pub device_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TwoFARequest {
    pub session_token: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}

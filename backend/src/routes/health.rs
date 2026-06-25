use axum::{Router, routing::get, Json};
use serde_json::json;

pub fn router() -> Router<crate::AppState> {
    Router::new().route("/", get(health_check))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "service": "tiketku-api", "version": env!("CARGO_PKG_VERSION") }))
}

use axum::{
    Router, Json,
    extract::State,
    routing::{get, put},
    Extension,
};
use serde_json::json;

use crate::{
    AppState,
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_settings).put(update_settings))
}

async fn get_settings(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let cache_key = "settings:app";
    if let Some(cached) = state.cache.get::<serde_json::Value>(cache_key).await {
        return Ok(Json(cached));
    }

    let rows = sqlx::query!("SELECT key, value FROM settings")
        .fetch_all(&state.db.pool)
        .await?;

    let mut map = serde_json::Map::new();
    for row in rows {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&row.value) {
            map.insert(row.key, v);
        } else {
            map.insert(row.key, serde_json::Value::String(row.value));
        }
    }

    let response = json!({ "success": true, "data": map });
    let _ = state.cache.set(cache_key, &response, 300).await;
    Ok(Json(response))
}

async fn update_settings(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Akses ditolak".into()));
    }

    if let serde_json::Value::Object(map) = req {
        for (key, value) in map {
            let serialized = serde_json::to_string(&value).unwrap_or_default();
            sqlx::query!(
                r#"INSERT INTO settings (key, value, updated_at)
                   VALUES ($1, $2, NOW())
                   ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()"#,
                key, serialized
            )
            .execute(&state.db.pool)
            .await?;
        }
    }

    let _ = state.cache.del("settings:app").await;
    Ok(Json(json!({ "success": true, "data": null })))
}

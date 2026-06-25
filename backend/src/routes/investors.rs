use axum::{Router, Json, extract::State, routing::get, Extension};
use crate::{AppState, error::AppResult, middleware::auth::AuthUser};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_investors))
}

async fn list_investors(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = sqlx::query!(
        "SELECT id, name, email, phone, status FROM investors ORDER BY name"
    )
    .fetch_all(&state.db.pool)
    .await?;

    let data: Vec<serde_json::Value> = rows.into_iter().map(|r| json!({
        "id": r.id,
        "name": r.name,
        "email": r.email,
        "phone": r.phone,
        "status": r.status,
    })).collect();

    Ok(Json(json!({ "success": true, "data": data })))
}

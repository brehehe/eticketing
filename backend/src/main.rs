use axum::{
    Router,
    http::{HeaderValue, Method},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
    timeout::TimeoutLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::time::Duration;

mod config;
mod db;
mod cache;
mod error;
mod models;
mod routes;
mod middleware;
mod services;

use config::AppConfig;
use db::Database;
use cache::Cache;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub cache: Cache,
    pub config: Arc<AppConfig>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Tracing ───────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "tiketku_api=debug,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ── Config ────────────────────────────────────────────────
    dotenv::dotenv().ok();
    let config = Arc::new(AppConfig::from_env()?);

    // ── Database ──────────────────────────────────────────────
    let db = Database::connect(&config.database_url).await?;
    // Migrations are applied manually via infra/migrations/
    // db.run_migrations().await?;

    // ── Cache (Redis) ─────────────────────────────────────────
    let cache = Cache::connect(&config.redis_url).await?;

    let state = AppState { db, cache, config: config.clone() };

    // ── CORS ──────────────────────────────────────────────────
    let cors = CorsLayer::new()
        .allow_origin(config.allowed_origins.iter()
            .map(|o| o.parse::<HeaderValue>().unwrap())
            .collect::<Vec<_>>())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    // ── Router ────────────────────────────────────────────────
    let app = Router::new()
        .nest("/api", routes::build_router(state))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors);

    // ── Listen ────────────────────────────────────────────────
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Server running on http://{}", addr);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

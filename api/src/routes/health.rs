use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::services::AppServices;

pub async fn health_check(
    State(services): State<Arc<AppServices>>,
) -> Json<Value> {
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_one(&services.db)
        .await
        .is_ok();

    let cache_healthy = services.cache.exists("health_check").await.is_ok();

    let status = if db_healthy && cache_healthy {
        "healthy"
    } else {
        "degraded"
    };

    Json(json!({
        "status": status,
        "database": if db_healthy { "connected" } else { "disconnected" },
        "cache": if cache_healthy { "connected" } else { "disconnected" },
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

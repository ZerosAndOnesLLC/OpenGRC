use axum::Json;
use serde_json::{json, Value};

pub async fn list_evidence() -> Json<Value> {
    Json(json!({
        "message": "Evidence endpoint - To be implemented",
        "evidence": []
    }))
}

pub async fn get_evidence() -> Json<Value> {
    Json(json!({
        "message": "Get evidence endpoint - To be implemented"
    }))
}

pub async fn create_evidence() -> Json<Value> {
    Json(json!({
        "message": "Create evidence endpoint - To be implemented"
    }))
}

pub async fn update_evidence() -> Json<Value> {
    Json(json!({
        "message": "Update evidence endpoint - To be implemented"
    }))
}

pub async fn delete_evidence() -> Json<Value> {
    Json(json!({
        "message": "Delete evidence endpoint - To be implemented"
    }))
}

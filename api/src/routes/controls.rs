use axum::Json;
use serde_json::{json, Value};

pub async fn list_controls() -> Json<Value> {
    Json(json!({
        "message": "Controls endpoint - To be implemented",
        "controls": []
    }))
}

pub async fn get_control() -> Json<Value> {
    Json(json!({
        "message": "Get control endpoint - To be implemented"
    }))
}

pub async fn create_control() -> Json<Value> {
    Json(json!({
        "message": "Create control endpoint - To be implemented"
    }))
}

pub async fn update_control() -> Json<Value> {
    Json(json!({
        "message": "Update control endpoint - To be implemented"
    }))
}

pub async fn delete_control() -> Json<Value> {
    Json(json!({
        "message": "Delete control endpoint - To be implemented"
    }))
}

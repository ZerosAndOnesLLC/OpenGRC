use axum::Json;
use serde_json::{json, Value};

pub async fn list_vendors() -> Json<Value> {
    Json(json!({
        "message": "Vendors endpoint - To be implemented",
        "vendors": []
    }))
}

pub async fn get_vendor() -> Json<Value> {
    Json(json!({
        "message": "Get vendor endpoint - To be implemented"
    }))
}

pub async fn create_vendor() -> Json<Value> {
    Json(json!({
        "message": "Create vendor endpoint - To be implemented"
    }))
}

pub async fn update_vendor() -> Json<Value> {
    Json(json!({
        "message": "Update vendor endpoint - To be implemented"
    }))
}

pub async fn delete_vendor() -> Json<Value> {
    Json(json!({
        "message": "Delete vendor endpoint - To be implemented"
    }))
}

use axum::Json;
use serde_json::{json, Value};

pub async fn list_audits() -> Json<Value> {
    Json(json!({
        "message": "Audits endpoint - To be implemented",
        "audits": []
    }))
}

pub async fn get_audit() -> Json<Value> {
    Json(json!({
        "message": "Get audit endpoint - To be implemented"
    }))
}

pub async fn create_audit() -> Json<Value> {
    Json(json!({
        "message": "Create audit endpoint - To be implemented"
    }))
}

pub async fn update_audit() -> Json<Value> {
    Json(json!({
        "message": "Update audit endpoint - To be implemented"
    }))
}

pub async fn delete_audit() -> Json<Value> {
    Json(json!({
        "message": "Delete audit endpoint - To be implemented"
    }))
}

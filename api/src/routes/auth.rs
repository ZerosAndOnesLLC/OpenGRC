use axum::{extract::Request, Json};
use serde_json::{json, Value};
use crate::middleware::get_auth_user;
use crate::utils::AppResult;

pub async fn me(request: Request) -> AppResult<Json<Value>> {
    let user = get_auth_user(&request)?;

    Ok(Json(json!({
        "id": user.id,
        "email": user.email,
        "organization_id": user.organization_id,
        "roles": user.roles,
    })))
}

use crate::utils::{AppError, AppResult};
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub organization_id: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Clone)]
pub struct AuthState {
    pub tv_api_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub client: reqwest::Client,
}

impl AuthState {
    pub fn new(tv_api_url: String, client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            tv_api_url,
            client_id,
            client_secret,
            redirect_uri,
            client: reqwest::Client::new(),
        }
    }
}

pub async fn auth_middleware(
    State(auth_state): State<Arc<AuthState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token(&headers)?;
    let user = validate_token(&auth_state, &token).await?;

    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

fn extract_token(headers: &HeaderMap) -> AppResult<String> {
    let auth_header = headers
        .get("Authorization")
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid Authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized(
            "Invalid Authorization header format".to_string(),
        ));
    }

    Ok(auth_header.trim_start_matches("Bearer ").to_string())
}

async fn validate_token(auth_state: &AuthState, token: &str) -> AppResult<AuthUser> {
    let url = format!("{}/userinfo", auth_state.tv_api_url.trim_end_matches('/'));

    let response = auth_state
        .client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to validate token with TitaniumVault: {:?}", e);
            AppError::Unauthorized("Failed to validate token".to_string())
        })?;

    if !response.status().is_success() {
        return Err(AppError::Unauthorized("Invalid token".to_string()));
    }

    let userinfo: serde_json::Value = response.json().await.map_err(|e| {
        tracing::error!("Failed to parse TitaniumVault response: {:?}", e);
        AppError::Unauthorized("Invalid token response".to_string())
    })?;

    // Extract user info from TV userinfo response
    let user = AuthUser {
        id: userinfo.get("sub")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        email: userinfo.get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        organization_id: userinfo.get("organization_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        roles: userinfo.get("roles")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(|| vec!["user".to_string()]),
    };

    Ok(user)
}

pub fn get_auth_user(request: &Request) -> AppResult<AuthUser> {
    request
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))
}

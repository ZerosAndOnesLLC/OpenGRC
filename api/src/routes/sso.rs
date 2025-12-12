use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::middleware::AuthState;
use crate::utils::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct ExchangeCodeRequest {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SSOValidationResponse {
    pub valid: bool,
    pub user: Option<SSOUser>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SSOUser {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: Option<i64>,
    pub iat: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateQuery {
    pub token: Option<String>,
}

/// POST /api/sso/exchange
/// Exchanges an authorization code for an access token
pub async fn exchange_code(
    State(auth_state): State<Arc<AuthState>>,
    Json(payload): Json<ExchangeCodeRequest>,
) -> AppResult<Json<TokenResponse>> {
    tracing::info!("Exchanging authorization code for access token");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::BadRequest(format!("Failed to create HTTP client: {}", e)))?;

    let token_endpoint = format!("{}/oauth/token", auth_state.tv_api_url.trim_end_matches('/'));
    tracing::debug!("Calling token endpoint: {}", token_endpoint);

    let form_params = [
        ("grant_type", "authorization_code"),
        ("code", &payload.code),
        ("client_id", &auth_state.client_id),
        ("client_secret", &auth_state.client_secret),
        ("redirect_uri", &auth_state.redirect_uri),
    ];

    tracing::debug!("Token request payload: client_id={}, code={}", auth_state.client_id, payload.code);

    let response = client
        .post(&token_endpoint)
        .form(&form_params)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Network error during token exchange: {}", e);
            AppError::BadRequest(format!("Failed to exchange code: {}", e))
        })?;

    let status = response.status();
    tracing::debug!("Token exchange response status: {}", status);

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Token exchange failed with status {}: {}", status, error_text);
        return Err(AppError::BadRequest(format!("Token exchange failed: {}", error_text)));
    }

    let token_response: TokenResponse = response
        .json()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to parse token response: {}", e)))?;

    tracing::info!("Successfully exchanged authorization code for access token");

    Ok(Json(token_response))
}

/// POST /api/sso/userinfo
/// Proxies the /userinfo request to TitaniumVault to avoid CORS issues
pub async fn get_userinfo(
    State(auth_state): State<Arc<AuthState>>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    tracing::info!("Proxying userinfo request to TitaniumVault");

    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::BadRequest(format!("Failed to create HTTP client: {}", e)))?;

    let userinfo_endpoint = format!("{}/userinfo", auth_state.tv_api_url.trim_end_matches('/'));
    tracing::debug!("Calling userinfo endpoint: {}", userinfo_endpoint);

    let response = client
        .get(&userinfo_endpoint)
        .header("Authorization", auth_header)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Network error during userinfo request: {}", e);
            AppError::BadRequest(format!("Failed to fetch userinfo: {}", e))
        })?;

    let status = response.status();
    tracing::debug!("Userinfo response status: {}", status);

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Userinfo request failed with status {}: {}", status, error_text);
        return Err(AppError::BadRequest(format!("Userinfo request failed: {}", error_text)));
    }

    let userinfo_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to parse userinfo response: {}", e)))?;

    tracing::info!("Successfully fetched userinfo from TitaniumVault");

    Ok(Json(userinfo_data))
}

/// GET /api/sso/validate
/// Validates SSO token from query parameter
pub async fn validate_sso(
    State(auth_state): State<Arc<AuthState>>,
    Query(query): Query<ValidateQuery>,
    headers: HeaderMap,
) -> AppResult<Json<SSOValidationResponse>> {
    let sso_token = if let Some(token) = query.token {
        token
    } else if let Some(auth_header) = headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        if auth_header.starts_with("Bearer ") {
            auth_header.trim_start_matches("Bearer ").to_string()
        } else {
            return Ok(Json(SSOValidationResponse {
                valid: false,
                user: None,
                error: Some("Invalid authorization header format".to_string()),
            }));
        }
    } else {
        return Ok(Json(SSOValidationResponse {
            valid: false,
            user: None,
            error: Some("No SSO token found".to_string()),
        }));
    };

    tracing::debug!("Validating SSO token: {}...", &sso_token[..std::cmp::min(10, sso_token.len())]);

    match validate_token_with_tv(&sso_token, &auth_state.tv_api_url).await {
        Ok(user) => {
            tracing::info!("SSO validation successful for user: {}", user.email);
            Ok(Json(SSOValidationResponse {
                valid: true,
                user: Some(user),
                error: None,
            }))
        }
        Err(e) => {
            tracing::warn!("SSO validation failed: {:?}", e);
            Ok(Json(SSOValidationResponse {
                valid: false,
                user: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

/// POST /api/sso/logout
/// Returns success (cookie clearing handled by client)
pub async fn logout_sso() -> AppResult<Json<serde_json::Value>> {
    tracing::debug!("Processing SSO logout request");

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "SSO logout successful"
    })))
}

async fn validate_token_with_tv(token: &str, tv_api_url: &str) -> Result<SSOUser, AppError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::BadRequest(format!("Failed to create HTTP client: {}", e)))?;

    let userinfo_endpoint = format!("{}/userinfo", tv_api_url.trim_end_matches('/'));
    tracing::debug!("Validating SSO token with endpoint: {}", userinfo_endpoint);

    let response = client
        .get(&userinfo_endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Network error during SSO token validation: {}", e);
            AppError::BadRequest(format!("Failed to validate token: {}", e))
        })?;

    let status = response.status();
    tracing::debug!("SSO token validation response status: {}", status);

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("SSO token validation failed with status {}: {}", status, error_text);
        return Err(AppError::Unauthorized("Invalid token".to_string()));
    }

    let userinfo: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to parse userinfo response: {}", e)))?;

    tracing::debug!("Received userinfo for SSO: {:?}", userinfo);

    let sso_user = SSOUser {
        sub: userinfo.get("sub")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        email: userinfo.get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        role: userinfo.get("role")
            .and_then(|v| v.as_str())
            .or_else(|| {
                userinfo.get("roles")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("user")
            .to_string(),
        exp: userinfo.get("exp").and_then(|v| v.as_i64()),
        iat: userinfo.get("iat").and_then(|v| v.as_i64()),
    };

    Ok(sso_user)
}

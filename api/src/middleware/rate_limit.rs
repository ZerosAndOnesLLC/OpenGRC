use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Duration;

use crate::cache::CacheClient;
use crate::utils::{AppError, AppResult};

/// Rate limiting configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: i32,
    pub requests_per_hour: i32,
    pub burst_size: i32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            requests_per_hour: 1000,
            burst_size: 10,
        }
    }
}

/// Rate limiter using Redis/Valkey for distributed rate limiting
#[derive(Clone)]
pub struct RateLimiter {
    cache: Arc<CacheClient>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(cache: Arc<CacheClient>, config: RateLimitConfig) -> Self {
        Self { cache, config }
    }

    /// Check if a request should be rate limited
    /// Returns (is_allowed, remaining_minute, remaining_hour, reset_timestamp)
    pub async fn check(&self, identifier: &str) -> AppResult<RateLimitResult> {
        let now = chrono::Utc::now();
        let minute_window = now.format("%Y%m%d%H%M").to_string();
        let hour_window = now.format("%Y%m%d%H").to_string();

        let minute_key = format!("ratelimit:{}:min:{}", identifier, minute_window);
        let hour_key = format!("ratelimit:{}:hr:{}", identifier, hour_window);

        // Increment counters
        let minute_count = self.cache.increment(&minute_key).await?;
        let hour_count = self.cache.increment(&hour_key).await?;

        // Set expiry on first request
        if minute_count == 1 {
            self.cache.expire(&minute_key, Duration::from_secs(60)).await?;
        }
        if hour_count == 1 {
            self.cache.expire(&hour_key, Duration::from_secs(3600)).await?;
        }

        let is_allowed = minute_count <= self.config.requests_per_minute as i64
            && hour_count <= self.config.requests_per_hour as i64;

        let remaining_minute = (self.config.requests_per_minute as i64 - minute_count).max(0);
        let remaining_hour = (self.config.requests_per_hour as i64 - hour_count).max(0);

        // Calculate reset times
        let reset_minute = (now + chrono::Duration::seconds(60 - now.timestamp() % 60)).timestamp();
        let reset_hour = (now + chrono::Duration::seconds(3600 - now.timestamp() % 3600)).timestamp();

        Ok(RateLimitResult {
            is_allowed,
            remaining_minute,
            remaining_hour,
            reset_minute,
            reset_hour,
            limit_minute: self.config.requests_per_minute,
            limit_hour: self.config.requests_per_hour,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub is_allowed: bool,
    pub remaining_minute: i64,
    pub remaining_hour: i64,
    pub reset_minute: i64,
    pub reset_hour: i64,
    pub limit_minute: i32,
    pub limit_hour: i32,
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Get identifier (prefer org_id from auth, fallback to IP)
    let identifier = get_rate_limit_identifier(&headers, &request);

    let result = limiter.check(&identifier).await?;

    if !result.is_allowed {
        return Err(AppError::TooManyRequests(format!(
            "Rate limit exceeded. Try again in {} seconds",
            result.reset_minute - chrono::Utc::now().timestamp()
        )));
    }

    // Execute the request
    let mut response = next.run(request).await;

    // Add rate limit headers to response
    let headers = response.headers_mut();
    headers.insert(
        "X-RateLimit-Limit-Minute",
        result.limit_minute.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Limit-Hour",
        result.limit_hour.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining-Minute",
        result.remaining_minute.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining-Hour",
        result.remaining_hour.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Reset",
        result.reset_minute.to_string().parse().unwrap(),
    );

    Ok(response)
}

/// Get the identifier for rate limiting
/// Priority: organization_id > API key > IP address
fn get_rate_limit_identifier(headers: &HeaderMap, _request: &Request) -> String {
    // Check for API key
    if let Some(auth) = headers.get("Authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ogrc_") {
                // Extract API key prefix for rate limiting
                let key = auth_str.trim_start_matches("Bearer ");
                if key.len() > 20 {
                    return format!("apikey:{}", &key[..20]);
                }
            }
        }
    }

    // Fallback to IP address
    get_client_ip(headers).unwrap_or_else(|| "unknown".to_string())
}

/// Extract client IP from headers (supports proxies like ALB, CloudFront)
fn get_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try X-Forwarded-For first (standard for AWS ALB, CloudFront)
    if let Some(xff) = headers.get("X-Forwarded-For") {
        if let Ok(xff_str) = xff.to_str() {
            // X-Forwarded-For can contain multiple IPs, take the first one
            if let Some(ip) = xff_str.split(',').next() {
                return Some(ip.trim().to_string());
            }
        }
    }

    // Try X-Real-IP (common alternative)
    if let Some(real_ip) = headers.get("X-Real-IP") {
        if let Ok(ip) = real_ip.to_str() {
            return Some(ip.to_string());
        }
    }

    // Try CF-Connecting-IP (Cloudflare)
    if let Some(cf_ip) = headers.get("CF-Connecting-IP") {
        if let Ok(ip) = cf_ip.to_str() {
            return Some(ip.to_string());
        }
    }

    None
}

/// Tier-based rate limiting configuration
pub fn get_rate_limit_config_for_tier(tier: &str) -> RateLimitConfig {
    match tier.to_lowercase().as_str() {
        "free" => RateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: 500,
            burst_size: 10,
        },
        "pro" => RateLimitConfig {
            requests_per_minute: 300,
            requests_per_hour: 3000,
            burst_size: 30,
        },
        "enterprise" => RateLimitConfig {
            requests_per_minute: 1000,
            requests_per_hour: 10000,
            burst_size: 100,
        },
        _ => RateLimitConfig::default(),
    }
}

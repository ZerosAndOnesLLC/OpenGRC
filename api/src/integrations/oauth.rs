//! OAuth2 Service for Integration Authentication
//!
//! Handles OAuth2 flows for integrations that support it:
//! - GitHub, GitLab, Okta, Google Workspace, Azure AD, Jira, etc.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::models::OAuthTokenResponse;

/// OAuth2 provider configuration
#[derive(Debug, Clone)]
pub struct OAuthProviderEndpoints {
    pub authorization_url: String,
    pub token_url: String,
    pub userinfo_url: Option<String>,
    pub revoke_url: Option<String>,
}

/// Configuration for an OAuth provider
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub endpoints: OAuthProviderEndpoints,
    pub default_scopes: Vec<String>,
    pub pkce_required: bool,
}

/// OAuth service for managing OAuth flows
#[derive(Clone)]
pub struct OAuthService {
    providers: HashMap<String, OAuthConfig>,
    pub redirect_base_url: String,
}

impl OAuthService {
    pub fn new(redirect_base_url: String) -> Self {
        Self {
            providers: HashMap::new(),
            redirect_base_url,
        }
    }

    /// Initialize with provider configurations from environment variables
    pub fn from_env(redirect_base_url: String) -> Self {
        let mut service = Self::new(redirect_base_url);

        // GitHub
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("GITHUB_OAUTH_CLIENT_ID"),
            std::env::var("GITHUB_OAUTH_CLIENT_SECRET"),
        ) {
            service.register_provider(
                "github",
                OAuthConfig {
                    client_id,
                    client_secret,
                    endpoints: OAuthProviderEndpoints {
                        authorization_url: "https://github.com/login/oauth/authorize".to_string(),
                        token_url: "https://github.com/login/oauth/access_token".to_string(),
                        userinfo_url: Some("https://api.github.com/user".to_string()),
                        revoke_url: None,
                    },
                    default_scopes: vec![
                        "read:user".to_string(),
                        "read:org".to_string(),
                        "repo".to_string(),
                        "security_events".to_string(),
                    ],
                    pkce_required: false,
                },
            );
        }

        // GitLab
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("GITLAB_OAUTH_CLIENT_ID"),
            std::env::var("GITLAB_OAUTH_CLIENT_SECRET"),
        ) {
            service.register_provider(
                "gitlab",
                OAuthConfig {
                    client_id,
                    client_secret,
                    endpoints: OAuthProviderEndpoints {
                        authorization_url: "https://gitlab.com/oauth/authorize".to_string(),
                        token_url: "https://gitlab.com/oauth/token".to_string(),
                        userinfo_url: Some("https://gitlab.com/api/v4/user".to_string()),
                        revoke_url: Some("https://gitlab.com/oauth/revoke".to_string()),
                    },
                    default_scopes: vec![
                        "read_user".to_string(),
                        "read_api".to_string(),
                        "read_repository".to_string(),
                    ],
                    pkce_required: true,
                },
            );
        }

        // Google (GCP / Google Workspace)
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("GOOGLE_OAUTH_CLIENT_ID"),
            std::env::var("GOOGLE_OAUTH_CLIENT_SECRET"),
        ) {
            // GCP
            service.register_provider(
                "gcp",
                OAuthConfig {
                    client_id: client_id.clone(),
                    client_secret: client_secret.clone(),
                    endpoints: OAuthProviderEndpoints {
                        authorization_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                        token_url: "https://oauth2.googleapis.com/token".to_string(),
                        userinfo_url: Some("https://www.googleapis.com/oauth2/v2/userinfo".to_string()),
                        revoke_url: Some("https://oauth2.googleapis.com/revoke".to_string()),
                    },
                    default_scopes: vec![
                        "https://www.googleapis.com/auth/cloud-platform.read-only".to_string(),
                        "https://www.googleapis.com/auth/cloudplatformprojects.readonly".to_string(),
                    ],
                    pkce_required: true,
                },
            );

            // Google Workspace
            service.register_provider(
                "google_workspace",
                OAuthConfig {
                    client_id,
                    client_secret,
                    endpoints: OAuthProviderEndpoints {
                        authorization_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                        token_url: "https://oauth2.googleapis.com/token".to_string(),
                        userinfo_url: Some("https://www.googleapis.com/oauth2/v2/userinfo".to_string()),
                        revoke_url: Some("https://oauth2.googleapis.com/revoke".to_string()),
                    },
                    default_scopes: vec![
                        "https://www.googleapis.com/auth/admin.directory.user.readonly".to_string(),
                        "https://www.googleapis.com/auth/admin.directory.group.readonly".to_string(),
                        "https://www.googleapis.com/auth/admin.reports.audit.readonly".to_string(),
                    ],
                    pkce_required: true,
                },
            );
        }

        // Azure AD (also used for Azure)
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("AZURE_OAUTH_CLIENT_ID"),
            std::env::var("AZURE_OAUTH_CLIENT_SECRET"),
        ) {
            let tenant = std::env::var("AZURE_OAUTH_TENANT_ID").unwrap_or_else(|_| "common".to_string());

            // Azure AD / Entra ID
            service.register_provider(
                "azure_ad",
                OAuthConfig {
                    client_id: client_id.clone(),
                    client_secret: client_secret.clone(),
                    endpoints: OAuthProviderEndpoints {
                        authorization_url: format!(
                            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
                            tenant
                        ),
                        token_url: format!(
                            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                            tenant
                        ),
                        userinfo_url: Some("https://graph.microsoft.com/v1.0/me".to_string()),
                        revoke_url: None,
                    },
                    default_scopes: vec!["https://graph.microsoft.com/.default".to_string()],
                    pkce_required: true,
                },
            );

            // Azure (same OAuth config)
            service.register_provider(
                "azure",
                OAuthConfig {
                    client_id,
                    client_secret,
                    endpoints: OAuthProviderEndpoints {
                        authorization_url: format!(
                            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
                            tenant
                        ),
                        token_url: format!(
                            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                            tenant
                        ),
                        userinfo_url: Some("https://graph.microsoft.com/v1.0/me".to_string()),
                        revoke_url: None,
                    },
                    default_scopes: vec!["https://management.azure.com/.default".to_string()],
                    pkce_required: true,
                },
            );
        }

        // Okta (requires domain in state/callback)
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("OKTA_OAUTH_CLIENT_ID"),
            std::env::var("OKTA_OAUTH_CLIENT_SECRET"),
        ) {
            // Okta requires domain-specific URLs, so we use a placeholder
            // The actual URLs will be constructed at runtime based on the domain in metadata
            service.register_provider(
                "okta",
                OAuthConfig {
                    client_id,
                    client_secret,
                    endpoints: OAuthProviderEndpoints {
                        authorization_url: "https://{domain}/oauth2/v1/authorize".to_string(),
                        token_url: "https://{domain}/oauth2/v1/token".to_string(),
                        userinfo_url: Some("https://{domain}/oauth2/v1/userinfo".to_string()),
                        revoke_url: Some("https://{domain}/oauth2/v1/revoke".to_string()),
                    },
                    default_scopes: vec![
                        "openid".to_string(),
                        "profile".to_string(),
                        "okta.users.read".to_string(),
                        "okta.groups.read".to_string(),
                        "okta.apps.read".to_string(),
                        "okta.logs.read".to_string(),
                    ],
                    pkce_required: true,
                },
            );
        }

        // Jira / Atlassian
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("ATLASSIAN_OAUTH_CLIENT_ID"),
            std::env::var("ATLASSIAN_OAUTH_CLIENT_SECRET"),
        ) {
            service.register_provider(
                "jira",
                OAuthConfig {
                    client_id,
                    client_secret,
                    endpoints: OAuthProviderEndpoints {
                        authorization_url: "https://auth.atlassian.com/authorize".to_string(),
                        token_url: "https://auth.atlassian.com/oauth/token".to_string(),
                        userinfo_url: Some("https://api.atlassian.com/me".to_string()),
                        revoke_url: None,
                    },
                    default_scopes: vec![
                        "read:jira-work".to_string(),
                        "read:jira-user".to_string(),
                        "offline_access".to_string(),
                    ],
                    pkce_required: true,
                },
            );
        }

        service
    }

    /// Register a provider configuration
    pub fn register_provider(&mut self, integration_type: &str, config: OAuthConfig) {
        self.providers.insert(integration_type.to_string(), config);
    }

    /// Check if OAuth is configured for an integration type
    pub fn is_configured(&self, integration_type: &str) -> bool {
        self.providers.contains_key(integration_type)
    }

    /// Get provider configuration
    pub fn get_provider(&self, integration_type: &str) -> Option<&OAuthConfig> {
        self.providers.get(integration_type)
    }

    /// Generate authorization URL
    pub fn generate_auth_url(
        &self,
        integration_type: &str,
        state: &str,
        code_verifier: Option<&str>,
        scopes: Option<&[String]>,
        extra_params: Option<&serde_json::Value>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<String, String> {
        let config = self
            .providers
            .get(integration_type)
            .ok_or_else(|| format!("OAuth not configured for {}", integration_type))?;

        let redirect_uri = format!(
            "{}/api/v1/integrations/oauth/callback",
            self.redirect_base_url
        );

        let scopes_str = scopes
            .map(|s| s.join(" "))
            .unwrap_or_else(|| config.default_scopes.join(" "));

        let mut auth_url = config.endpoints.authorization_url.clone();

        // Handle Okta domain substitution
        if integration_type == "okta" {
            if let Some(meta) = metadata {
                if let Some(domain) = meta.get("domain").and_then(|d| d.as_str()) {
                    auth_url = auth_url.replace("{domain}", domain);
                } else {
                    return Err("Okta domain required in metadata".to_string());
                }
            } else {
                return Err("Okta domain required in metadata".to_string());
            }
        }

        let mut url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&state={}&scope={}",
            auth_url,
            urlencoding::encode(&config.client_id),
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(state),
            urlencoding::encode(&scopes_str)
        );

        // Add PKCE code challenge if required
        if config.pkce_required {
            if let Some(verifier) = code_verifier {
                let challenge = generate_code_challenge(verifier);
                url.push_str(&format!(
                    "&code_challenge={}&code_challenge_method=S256",
                    urlencoding::encode(&challenge)
                ));
            }
        }

        // Add extra params (like access_type=offline for Google)
        if let Some(extra) = extra_params {
            if let Some(obj) = extra.as_object() {
                for (key, value) in obj {
                    if let Some(v) = value.as_str() {
                        url.push_str(&format!("&{}={}", key, urlencoding::encode(v)));
                    }
                }
            }
        }

        Ok(url)
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(
        &self,
        integration_type: &str,
        code: &str,
        code_verifier: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<OAuthTokenResponse, String> {
        let config = self
            .providers
            .get(integration_type)
            .ok_or_else(|| format!("OAuth not configured for {}", integration_type))?;

        let redirect_uri = format!(
            "{}/api/v1/integrations/oauth/callback",
            self.redirect_base_url
        );

        let mut token_url = config.endpoints.token_url.clone();

        // Handle Okta domain substitution
        if integration_type == "okta" {
            if let Some(meta) = metadata {
                if let Some(domain) = meta.get("domain").and_then(|d| d.as_str()) {
                    token_url = token_url.replace("{domain}", domain);
                } else {
                    return Err("Okta domain required in metadata".to_string());
                }
            } else {
                return Err("Okta domain required in metadata".to_string());
            }
        }

        let client = reqwest::Client::new();
        let mut params = vec![
            ("grant_type", "authorization_code".to_string()),
            ("code", code.to_string()),
            ("redirect_uri", redirect_uri),
            ("client_id", config.client_id.clone()),
            ("client_secret", config.client_secret.clone()),
        ];

        if config.pkce_required {
            if let Some(verifier) = code_verifier {
                params.push(("code_verifier", verifier.to_string()));
            }
        }

        let mut request = client.post(&token_url).form(&params);

        // GitHub requires Accept header for JSON response
        if integration_type == "github" {
            request = request.header("Accept", "application/json");
        }

        let response = request.send().await.map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Token exchange failed: {}", error_text));
        }

        let token_response: OAuthTokenResponse =
            response.json().await.map_err(|e| e.to_string())?;

        Ok(token_response)
    }

    /// Refresh an access token
    pub async fn refresh_token(
        &self,
        integration_type: &str,
        refresh_token: &str,
        metadata: Option<&serde_json::Value>,
    ) -> Result<OAuthTokenResponse, String> {
        let config = self
            .providers
            .get(integration_type)
            .ok_or_else(|| format!("OAuth not configured for {}", integration_type))?;

        let mut token_url = config.endpoints.token_url.clone();

        // Handle Okta domain substitution
        if integration_type == "okta" {
            if let Some(meta) = metadata {
                if let Some(domain) = meta.get("domain").and_then(|d| d.as_str()) {
                    token_url = token_url.replace("{domain}", domain);
                } else {
                    return Err("Okta domain required in metadata".to_string());
                }
            } else {
                return Err("Okta domain required in metadata".to_string());
            }
        }

        let client = reqwest::Client::new();
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
        ];

        let mut request = client.post(&token_url).form(&params);

        // GitHub requires Accept header
        if integration_type == "github" {
            request = request.header("Accept", "application/json");
        }

        let response = request.send().await.map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Token refresh failed: {}", error_text));
        }

        let token_response: OAuthTokenResponse =
            response.json().await.map_err(|e| e.to_string())?;

        Ok(token_response)
    }

    /// Revoke a token (if supported)
    pub async fn revoke_token(
        &self,
        integration_type: &str,
        token: &str,
        metadata: Option<&serde_json::Value>,
    ) -> Result<(), String> {
        let config = self
            .providers
            .get(integration_type)
            .ok_or_else(|| format!("OAuth not configured for {}", integration_type))?;

        let revoke_url = match &config.endpoints.revoke_url {
            Some(url) => url.clone(),
            None => return Ok(()), // No revoke endpoint, just return success
        };

        let mut revoke_url = revoke_url;

        // Handle Okta domain substitution
        if integration_type == "okta" {
            if let Some(meta) = metadata {
                if let Some(domain) = meta.get("domain").and_then(|d| d.as_str()) {
                    revoke_url = revoke_url.replace("{domain}", domain);
                }
            }
        }

        let client = reqwest::Client::new();
        let params = [
            ("token", token),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
        ];

        let response = client
            .post(&revoke_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Token revocation failed: {}", error_text));
        }

        Ok(())
    }
}

/// Generate a random state parameter for CSRF protection
pub fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Generate a random code verifier for PKCE
pub fn generate_code_verifier() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Generate code challenge from code verifier (S256)
pub fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_state() {
        let state = generate_state();
        assert!(!state.is_empty());
        assert!(state.len() > 20); // Should be reasonably long
    }

    #[test]
    fn test_generate_code_verifier() {
        let verifier = generate_code_verifier();
        assert!(!verifier.is_empty());
        assert!(verifier.len() > 20);
    }

    #[test]
    fn test_generate_code_challenge() {
        let verifier = "test_verifier_string";
        let challenge = generate_code_challenge(verifier);
        assert!(!challenge.is_empty());
        // Same verifier should produce same challenge
        assert_eq!(challenge, generate_code_challenge(verifier));
    }
}

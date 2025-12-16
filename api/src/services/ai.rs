use crate::cache::{org_cache_key, CacheClient};
use crate::utils::{AppError, AppResult, EncryptionService};
use chrono::{DateTime, Utc};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use std::time::{Duration, Instant};
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour for AI responses
const CACHE_PREFIX_AI: &str = "ai";

// ==================== Models ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiConfiguration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub provider: String,
    pub api_endpoint: Option<String>,
    #[sqlx(skip)]
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
    pub model: String,
    pub max_tokens: Option<i32>,
    pub temperature: Option<Decimal>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAiConfiguration {
    pub provider: String,
    pub api_endpoint: Option<String>,
    pub api_key: String,
    pub model: Option<String>,
    pub max_tokens: Option<i32>,
    pub temperature: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiCompletion {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub feature: String,
    pub prompt_hash: String,
    pub input_context: serde_json::Value,
    pub prompt_text: String,
    pub completion_text: Option<String>,
    pub model: Option<String>,
    pub tokens_input: Option<i32>,
    pub tokens_output: Option<i32>,
    pub latency_ms: Option<i32>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiPolicyDraft {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub completion_id: Option<Uuid>,
    pub title: String,
    pub category: Option<String>,
    pub framework_codes: Option<Vec<String>>,
    pub generated_content: String,
    pub user_prompt: Option<String>,
    pub accepted: bool,
    pub accepted_policy_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiEvidenceSummary {
    pub id: Uuid,
    pub evidence_id: Uuid,
    pub completion_id: Option<Uuid>,
    pub summary: String,
    pub key_points: Option<serde_json::Value>,
    pub compliance_relevance: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiGapRecommendation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub framework_id: Uuid,
    pub completion_id: Option<Uuid>,
    pub requirement_id: Option<Uuid>,
    pub gap_description: String,
    pub recommendation: String,
    pub priority: Option<String>,
    pub estimated_effort: Option<String>,
    pub suggested_controls: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiRiskAssessment {
    pub id: Uuid,
    pub risk_id: Uuid,
    pub completion_id: Option<Uuid>,
    pub suggested_likelihood: Option<i32>,
    pub suggested_impact: Option<i32>,
    pub likelihood_rationale: Option<String>,
    pub impact_rationale: Option<String>,
    pub suggested_treatment: Option<String>,
    pub control_recommendations: Option<serde_json::Value>,
    pub accepted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiAuditPreparation {
    pub id: Uuid,
    pub audit_id: Uuid,
    pub completion_id: Option<Uuid>,
    pub preparation_summary: String,
    pub checklist_items: serde_json::Value,
    pub evidence_gaps: Option<serde_json::Value>,
    pub risk_areas: Option<serde_json::Value>,
    pub timeline_suggestions: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== Request/Response Types ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyDraftRequest {
    pub title: String,
    pub category: Option<String>,
    pub description: String,
    pub framework_codes: Option<Vec<String>>,
    pub additional_context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceSummaryRequest {
    pub evidence_id: Uuid,
    pub evidence_title: String,
    pub evidence_description: Option<String>,
    pub evidence_content: Option<String>,
    pub control_codes: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GapAnalysisRequest {
    pub framework_id: Uuid,
    pub include_recommendations: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskScoringRequest {
    pub risk_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NaturalLanguageSearchRequest {
    pub query: String,
    pub scope: Option<Vec<String>>,  // controls, evidence, policies, risks
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NaturalLanguageSearchResult {
    pub interpreted_query: String,
    pub search_filters: serde_json::Value,
    pub results: Vec<SearchResultItem>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub relevance_score: f32,
    pub match_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditPrepRequest {
    pub audit_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiStats {
    pub total_completions: i64,
    pub completions_this_month: i64,
    pub tokens_used_this_month: i64,
    pub by_feature: Vec<FeatureUsage>,
    pub ai_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FeatureUsage {
    pub feature: String,
    pub count: i64,
}

// ==================== LLM API Types ====================

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: i32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: i32,
    output_tokens: i32,
}

// ==================== Service ====================

#[derive(Clone)]
pub struct AiService {
    db: PgPool,
    cache: CacheClient,
    encryption: EncryptionService,
    http_client: Client,
}

impl AiService {
    pub fn new(db: PgPool, cache: CacheClient, encryption: EncryptionService) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            db,
            cache,
            encryption,
            http_client,
        }
    }

    // ==================== Configuration ====================

    pub async fn get_configuration(&self, org_id: Uuid) -> AppResult<Option<AiConfiguration>> {
        let config: Option<AiConfiguration> = sqlx::query_as(
            r#"
            SELECT id, organization_id, provider, api_endpoint, model, max_tokens,
                   temperature, enabled, created_at, updated_at
            FROM ai_configurations
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(config)
    }

    pub async fn save_configuration(
        &self,
        org_id: Uuid,
        input: CreateAiConfiguration,
    ) -> AppResult<AiConfiguration> {
        let encrypted_key = self.encryption.encrypt(&input.api_key)
            .map_err(|e| AppError::InternalServerError(format!("Failed to encrypt API key: {}", e)))?;

        let config = sqlx::query_as::<_, AiConfiguration>(
            r#"
            INSERT INTO ai_configurations (organization_id, provider, api_endpoint, api_key_encrypted, model, max_tokens, temperature)
            VALUES ($1, $2, $3, $4, COALESCE($5, 'gpt-4o-mini'), $6, $7)
            ON CONFLICT (organization_id) DO UPDATE SET
                provider = EXCLUDED.provider,
                api_endpoint = EXCLUDED.api_endpoint,
                api_key_encrypted = EXCLUDED.api_key_encrypted,
                model = EXCLUDED.model,
                max_tokens = COALESCE(EXCLUDED.max_tokens, ai_configurations.max_tokens),
                temperature = COALESCE(EXCLUDED.temperature, ai_configurations.temperature),
                updated_at = NOW()
            RETURNING id, organization_id, provider, api_endpoint, model, max_tokens,
                      temperature, enabled, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(&input.provider)
        .bind(&input.api_endpoint)
        .bind(&encrypted_key)
        .bind(&input.model)
        .bind(input.max_tokens)
        .bind(input.temperature)
        .fetch_one(&self.db)
        .await?;

        Ok(config)
    }

    pub async fn toggle_ai(&self, org_id: Uuid, enabled: bool) -> AppResult<()> {
        sqlx::query("UPDATE ai_configurations SET enabled = $1, updated_at = NOW() WHERE organization_id = $2")
            .bind(enabled)
            .bind(org_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    // ==================== Core LLM Interaction ====================

    async fn get_api_key(&self, org_id: Uuid) -> AppResult<(String, AiConfiguration)> {
        let config = self.get_configuration(org_id).await?
            .ok_or_else(|| AppError::BadRequest("AI not configured for this organization".into()))?;

        if !config.enabled {
            return Err(AppError::BadRequest("AI features are disabled".into()));
        }

        let encrypted_key: String = sqlx::query_scalar(
            "SELECT api_key_encrypted FROM ai_configurations WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let api_key = self.encryption.decrypt(&encrypted_key)
            .map_err(|e| AppError::InternalServerError(format!("Failed to decrypt API key: {}", e)))?;

        Ok((api_key, config))
    }

    async fn call_llm(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        feature: &str,
        system_prompt: &str,
        user_prompt: &str,
        context: serde_json::Value,
    ) -> AppResult<(String, Uuid)> {
        let (api_key, config) = self.get_api_key(org_id).await?;

        // Create prompt hash for caching
        let prompt_hash = {
            let mut hasher = Sha256::new();
            hasher.update(system_prompt.as_bytes());
            hasher.update(user_prompt.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        // Check cache first
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_AI, &prompt_hash);
        if let Some(cached) = self.cache.get::<String>(&cache_key).await? {
            tracing::debug!("Cache hit for AI completion");
            // Return cached response with a placeholder completion ID
            let completion_id = Uuid::new_v4();
            return Ok((cached, completion_id));
        }

        let full_prompt = format!("{}\n\nUser Request:\n{}", system_prompt, user_prompt);

        // Record the completion attempt
        let completion_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO ai_completions (organization_id, user_id, feature, prompt_hash, input_context, prompt_text, status)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending')
            RETURNING id
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(feature)
        .bind(&prompt_hash)
        .bind(&context)
        .bind(&full_prompt)
        .fetch_one(&self.db)
        .await?;

        let start = Instant::now();

        let result = match config.provider.as_str() {
            "anthropic" => self.call_anthropic(&api_key, &config, system_prompt, user_prompt).await,
            _ => self.call_openai_compatible(&api_key, &config, system_prompt, user_prompt).await,
        };

        let latency_ms = start.elapsed().as_millis() as i32;

        match result {
            Ok((response, tokens_in, tokens_out)) => {
                // Update completion record
                sqlx::query(
                    r#"
                    UPDATE ai_completions
                    SET completion_text = $1, model = $2, tokens_input = $3, tokens_output = $4,
                        latency_ms = $5, status = 'completed'
                    WHERE id = $6
                    "#,
                )
                .bind(&response)
                .bind(&config.model)
                .bind(tokens_in)
                .bind(tokens_out)
                .bind(latency_ms)
                .bind(completion_id)
                .execute(&self.db)
                .await?;

                // Cache the response
                self.cache.set(&cache_key, &response, Some(CACHE_TTL)).await?;

                Ok((response, completion_id))
            }
            Err(e) => {
                // Record the error
                sqlx::query(
                    "UPDATE ai_completions SET status = 'error', error_message = $1, latency_ms = $2 WHERE id = $3",
                )
                .bind(e.to_string())
                .bind(latency_ms)
                .bind(completion_id)
                .execute(&self.db)
                .await?;

                Err(e)
            }
        }
    }

    async fn call_openai_compatible(
        &self,
        api_key: &str,
        config: &AiConfiguration,
        system_prompt: &str,
        user_prompt: &str,
    ) -> AppResult<(String, i32, i32)> {
        let endpoint = config.api_endpoint.as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");

        let request = OpenAiRequest {
            model: config.model.clone(),
            messages: vec![
                OpenAiMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OpenAiMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            max_tokens: config.max_tokens.unwrap_or(4096),
            temperature: config.temperature
                .map(|t| t.to_string().parse().unwrap_or(0.7))
                .unwrap_or(0.7),
        };

        let response = self.http_client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("LLM API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::InternalServerError(format!("LLM API error: {}", error_text)));
        }

        let result: OpenAiResponse = response.json().await
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse LLM response: {}", e)))?;

        let text = result.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let (tokens_in, tokens_out) = result.usage
            .map(|u| (u.prompt_tokens, u.completion_tokens))
            .unwrap_or((0, 0));

        Ok((text, tokens_in, tokens_out))
    }

    async fn call_anthropic(
        &self,
        api_key: &str,
        config: &AiConfiguration,
        system_prompt: &str,
        user_prompt: &str,
    ) -> AppResult<(String, i32, i32)> {
        let endpoint = config.api_endpoint.as_deref()
            .unwrap_or("https://api.anthropic.com/v1/messages");

        let full_user_message = format!("{}\n\n{}", system_prompt, user_prompt);

        let request = AnthropicRequest {
            model: config.model.clone(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: full_user_message,
            }],
            max_tokens: config.max_tokens.unwrap_or(4096),
        };

        let response = self.http_client
            .post(endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Anthropic API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::InternalServerError(format!("Anthropic API error: {}", error_text)));
        }

        let result: AnthropicResponse = response.json().await
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse Anthropic response: {}", e)))?;

        let text = result.content.first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        let (tokens_in, tokens_out) = result.usage
            .map(|u| (u.input_tokens, u.output_tokens))
            .unwrap_or((0, 0));

        Ok((text, tokens_in, tokens_out))
    }

    // ==================== Feature: Policy Drafting ====================

    pub async fn draft_policy(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        request: PolicyDraftRequest,
    ) -> AppResult<AiPolicyDraft> {
        let system_prompt = r#"You are an expert compliance policy writer. Generate professional, comprehensive security and compliance policies based on user requirements.

Output Format:
- Use Markdown formatting
- Include these sections: Purpose, Scope, Policy Statements (numbered), Roles & Responsibilities, Compliance, Exceptions, Related Documents, Definitions
- Be specific and actionable
- Include measurable requirements where possible
- Reference relevant compliance frameworks when applicable

Important: Generate ONLY the policy content in Markdown. Do not include any explanation or preamble."#;

        let mut user_prompt = format!(
            "Create a policy titled '{}' for the category '{}'.\n\nDescription: {}",
            request.title,
            request.category.as_deref().unwrap_or("security"),
            request.description
        );

        if let Some(frameworks) = &request.framework_codes {
            user_prompt.push_str(&format!("\n\nThis policy should address requirements from: {}", frameworks.join(", ")));
        }

        if let Some(context) = &request.additional_context {
            user_prompt.push_str(&format!("\n\nAdditional context: {}", context));
        }

        let context = serde_json::to_value(&request)
            .map_err(|e| AppError::InternalServerError(format!("Failed to serialize context: {}", e)))?;
        let (generated_content, completion_id) = self.call_llm(
            org_id,
            user_id,
            "policy_draft",
            system_prompt,
            &user_prompt,
            context,
        ).await?;

        // Save the draft
        let draft = sqlx::query_as::<_, AiPolicyDraft>(
            r#"
            INSERT INTO ai_policy_drafts (organization_id, completion_id, title, category, framework_codes, generated_content, user_prompt, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, organization_id, completion_id, title, category, framework_codes, generated_content, user_prompt, accepted, accepted_policy_id, created_by, created_at
            "#,
        )
        .bind(org_id)
        .bind(completion_id)
        .bind(&request.title)
        .bind(&request.category)
        .bind(&request.framework_codes)
        .bind(&generated_content)
        .bind(&request.description)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(draft)
    }

    pub async fn list_policy_drafts(&self, org_id: Uuid) -> AppResult<Vec<AiPolicyDraft>> {
        let drafts = sqlx::query_as::<_, AiPolicyDraft>(
            r#"
            SELECT id, organization_id, completion_id, title, category, framework_codes,
                   generated_content, user_prompt, accepted, accepted_policy_id, created_by, created_at
            FROM ai_policy_drafts
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT 50
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(drafts)
    }

    pub async fn accept_policy_draft(&self, org_id: Uuid, draft_id: Uuid, policy_id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE ai_policy_drafts SET accepted = true, accepted_policy_id = $1 WHERE id = $2 AND organization_id = $3",
        )
        .bind(policy_id)
        .bind(draft_id)
        .bind(org_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ==================== Feature: Evidence Summarization ====================

    pub async fn summarize_evidence(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        request: EvidenceSummaryRequest,
    ) -> AppResult<AiEvidenceSummary> {
        // Check for existing summary
        let existing: Option<AiEvidenceSummary> = sqlx::query_as(
            "SELECT * FROM ai_evidence_summaries WHERE evidence_id = $1",
        )
        .bind(request.evidence_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(summary) = existing {
            return Ok(summary);
        }

        let system_prompt = r#"You are a compliance evidence analyst. Analyze the provided evidence and generate a concise summary.

Output a JSON object with this structure:
{
  "summary": "A 2-3 sentence summary of what this evidence demonstrates",
  "key_points": ["Point 1", "Point 2", "Point 3"],
  "compliance_relevance": {
    "controls": ["Control codes this evidence supports"],
    "frameworks": ["Applicable frameworks"],
    "gaps": ["Any gaps or concerns identified"]
  }
}

Be specific about what compliance requirements this evidence helps satisfy."#;

        let user_prompt = format!(
            "Evidence Title: {}\nDescription: {}\nContent: {}\nLinked Controls: {}",
            request.evidence_title,
            request.evidence_description.as_deref().unwrap_or("N/A"),
            request.evidence_content.as_deref().unwrap_or("Content not provided"),
            request.control_codes.as_ref().map(|c| c.join(", ")).unwrap_or_else(|| "None".to_string())
        );

        let context = serde_json::to_value(&request).map_err(|e| AppError::InternalServerError(format!("Failed to serialize context: {}", e)))?;
        let (response, completion_id) = self.call_llm(
            org_id,
            user_id,
            "evidence_summary",
            system_prompt,
            &user_prompt,
            context,
        ).await?;

        // Parse the JSON response
        let parsed: serde_json::Value = serde_json::from_str(&response)
            .unwrap_or_else(|_| serde_json::json!({
                "summary": response,
                "key_points": [],
                "compliance_relevance": {}
            }));

        let summary_text = parsed["summary"].as_str().unwrap_or(&response).to_string();
        let key_points = parsed.get("key_points").cloned();
        let compliance_relevance = parsed.get("compliance_relevance").cloned();

        let summary = sqlx::query_as::<_, AiEvidenceSummary>(
            r#"
            INSERT INTO ai_evidence_summaries (evidence_id, completion_id, summary, key_points, compliance_relevance)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (evidence_id) DO UPDATE SET
                summary = EXCLUDED.summary,
                key_points = EXCLUDED.key_points,
                compliance_relevance = EXCLUDED.compliance_relevance,
                completion_id = EXCLUDED.completion_id
            RETURNING id, evidence_id, completion_id, summary, key_points, compliance_relevance, created_at
            "#,
        )
        .bind(request.evidence_id)
        .bind(completion_id)
        .bind(&summary_text)
        .bind(&key_points)
        .bind(&compliance_relevance)
        .fetch_one(&self.db)
        .await?;

        Ok(summary)
    }

    // ==================== Feature: Gap Analysis Recommendations ====================

    pub async fn get_gap_recommendations(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        request: GapAnalysisRequest,
    ) -> AppResult<Vec<AiGapRecommendation>> {
        // Get framework info and gap data
        let framework: (String, String) = sqlx::query_as(
            "SELECT name, COALESCE(description, '') FROM frameworks WHERE id = $1"
        )
        .bind(request.framework_id)
        .fetch_one(&self.db)
        .await?;

        // Get uncovered requirements
        let gaps: Vec<(Uuid, String, String, String)> = sqlx::query_as(
            r#"
            SELECT fr.id, fr.code, fr.name, COALESCE(fr.description, '')
            FROM framework_requirements fr
            WHERE fr.framework_id = $1
              AND NOT EXISTS (
                  SELECT 1 FROM control_requirement_mappings crm
                  JOIN controls c ON crm.control_id = c.id
                  WHERE crm.framework_requirement_id = fr.id
                    AND c.organization_id = $2
                    AND c.status IN ('implemented', 'partially_implemented')
              )
            ORDER BY fr.sort_order
            LIMIT 20
            "#,
        )
        .bind(request.framework_id)
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        if gaps.is_empty() {
            return Ok(vec![]);
        }

        let system_prompt = r#"You are a compliance gap analysis expert. For each requirement gap, provide actionable recommendations.

Output a JSON array where each item has:
{
  "requirement_code": "The requirement code",
  "gap_description": "What is missing or incomplete",
  "recommendation": "Specific steps to address the gap",
  "priority": "critical|high|medium|low",
  "estimated_effort": "hours|days|weeks",
  "suggested_controls": ["Control implementations that would address this"]
}

Be specific and practical in your recommendations."#;

        let gaps_text: Vec<String> = gaps.iter()
            .map(|(_, code, name, desc)| format!("- {} ({}): {}", code, name, desc))
            .collect();

        let user_prompt = format!(
            "Framework: {} ({})\n\nUncovered Requirements:\n{}",
            framework.0,
            framework.1,
            gaps_text.join("\n")
        );

        let context = serde_json::json!({
            "framework_id": request.framework_id,
            "gap_count": gaps.len()
        });

        let (response, completion_id) = self.call_llm(
            org_id,
            user_id,
            "gap_analysis",
            system_prompt,
            &user_prompt,
            context,
        ).await?;

        // Parse response and save recommendations
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&response)
            .unwrap_or_else(|_| vec![]);

        let mut recommendations = Vec::new();

        for (i, item) in parsed.iter().enumerate() {
            let req_code = item["requirement_code"].as_str().unwrap_or("");
            let requirement_id = gaps.iter()
                .find(|(_, code, _, _)| code == req_code)
                .map(|(id, _, _, _)| *id);

            if requirement_id.is_none() && i < gaps.len() {
                // Fall back to index-based matching
            }

            let rec = sqlx::query_as::<_, AiGapRecommendation>(
                r#"
                INSERT INTO ai_gap_recommendations
                (organization_id, framework_id, completion_id, requirement_id, gap_description, recommendation, priority, estimated_effort, suggested_controls)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id, organization_id, framework_id, completion_id, requirement_id, gap_description, recommendation, priority, estimated_effort, suggested_controls, created_at
                "#,
            )
            .bind(org_id)
            .bind(request.framework_id)
            .bind(completion_id)
            .bind(requirement_id.or_else(|| gaps.get(i).map(|(id, _, _, _)| *id)))
            .bind(item["gap_description"].as_str().unwrap_or("Gap identified"))
            .bind(item["recommendation"].as_str().unwrap_or("Review and implement controls"))
            .bind(item["priority"].as_str())
            .bind(item["estimated_effort"].as_str())
            .bind(item.get("suggested_controls"))
            .fetch_one(&self.db)
            .await?;

            recommendations.push(rec);
        }

        Ok(recommendations)
    }

    // ==================== Feature: Risk Scoring Suggestions ====================

    pub async fn suggest_risk_scoring(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        request: RiskScoringRequest,
    ) -> AppResult<AiRiskAssessment> {
        // Get risk details
        let risk: (String, String, Option<String>, Option<String>, Option<i32>, Option<i32>) = sqlx::query_as(
            r#"
            SELECT title, COALESCE(description, ''), category, source, likelihood, impact
            FROM risks WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(request.risk_id)
        .bind(org_id)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AppError::NotFound("Risk not found".into()))?;

        // Get existing controls for this risk
        let controls: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT c.code, c.name
            FROM controls c
            JOIN risk_control_mappings rcm ON c.id = rcm.control_id
            WHERE rcm.risk_id = $1
            "#,
        )
        .bind(request.risk_id)
        .fetch_all(&self.db)
        .await?;

        let system_prompt = r#"You are a risk assessment expert. Analyze the provided risk and suggest appropriate likelihood and impact scores.

Use a 1-5 scale where:
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

Output a JSON object:
{
  "suggested_likelihood": 1-5,
  "suggested_impact": 1-5,
  "likelihood_rationale": "Explanation for likelihood score",
  "impact_rationale": "Explanation for impact score",
  "suggested_treatment": "Recommended risk treatment approach",
  "control_recommendations": ["Suggested controls to mitigate this risk"]
}

Consider industry context, the nature of the risk, and existing controls when scoring."#;

        let controls_text = if controls.is_empty() {
            "None currently mapped".to_string()
        } else {
            controls.iter().map(|(c, n)| format!("{} ({})", c, n)).collect::<Vec<_>>().join(", ")
        };

        let user_prompt = format!(
            "Risk: {}\nDescription: {}\nCategory: {}\nSource: {}\nCurrent Scores: Likelihood={}, Impact={}\nExisting Controls: {}",
            risk.0,
            risk.1,
            risk.2.as_deref().unwrap_or("Unknown"),
            risk.3.as_deref().unwrap_or("Unknown"),
            risk.4.map(|l| l.to_string()).unwrap_or_else(|| "Not set".to_string()),
            risk.5.map(|i| i.to_string()).unwrap_or_else(|| "Not set".to_string()),
            controls_text
        );

        let context = serde_json::json!({
            "risk_id": request.risk_id,
            "current_likelihood": risk.4,
            "current_impact": risk.5
        });

        let (response, completion_id) = self.call_llm(
            org_id,
            user_id,
            "risk_scoring",
            system_prompt,
            &user_prompt,
            context,
        ).await?;

        let parsed: serde_json::Value = serde_json::from_str(&response)
            .unwrap_or_else(|_| serde_json::json!({}));

        let assessment = sqlx::query_as::<_, AiRiskAssessment>(
            r#"
            INSERT INTO ai_risk_assessments
            (risk_id, completion_id, suggested_likelihood, suggested_impact, likelihood_rationale, impact_rationale, suggested_treatment, control_recommendations)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, risk_id, completion_id, suggested_likelihood, suggested_impact, likelihood_rationale, impact_rationale, suggested_treatment, control_recommendations, accepted, created_at
            "#,
        )
        .bind(request.risk_id)
        .bind(completion_id)
        .bind(parsed["suggested_likelihood"].as_i64().map(|v| v as i32))
        .bind(parsed["suggested_impact"].as_i64().map(|v| v as i32))
        .bind(parsed["likelihood_rationale"].as_str())
        .bind(parsed["impact_rationale"].as_str())
        .bind(parsed["suggested_treatment"].as_str())
        .bind(parsed.get("control_recommendations"))
        .fetch_one(&self.db)
        .await?;

        Ok(assessment)
    }

    pub async fn accept_risk_assessment(&self, assessment_id: Uuid, risk_id: Uuid) -> AppResult<()> {
        // Get the assessment
        let assessment: AiRiskAssessment = sqlx::query_as(
            "SELECT * FROM ai_risk_assessments WHERE id = $1 AND risk_id = $2"
        )
        .bind(assessment_id)
        .bind(risk_id)
        .fetch_one(&self.db)
        .await?;

        // Update the risk with suggested scores
        if assessment.suggested_likelihood.is_some() || assessment.suggested_impact.is_some() {
            sqlx::query(
                r#"
                UPDATE risks SET
                    likelihood = COALESCE($1, likelihood),
                    impact = COALESCE($2, impact),
                    inherent_score = COALESCE($1, likelihood) * COALESCE($2, impact),
                    updated_at = NOW()
                WHERE id = $3
                "#,
            )
            .bind(assessment.suggested_likelihood)
            .bind(assessment.suggested_impact)
            .bind(risk_id)
            .execute(&self.db)
            .await?;
        }

        // Mark assessment as accepted
        sqlx::query("UPDATE ai_risk_assessments SET accepted = true WHERE id = $1")
            .bind(assessment_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    // ==================== Feature: Natural Language Search ====================

    pub async fn natural_language_search(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        request: NaturalLanguageSearchRequest,
    ) -> AppResult<NaturalLanguageSearchResult> {
        let system_prompt = r#"You are a compliance search assistant. Convert natural language queries into structured search parameters.

Output a JSON object:
{
  "interpreted_query": "What the user is looking for in plain terms",
  "entity_types": ["controls", "evidence", "policies", "risks"],
  "keywords": ["key", "search", "terms"],
  "filters": {
    "status": "optional status filter",
    "category": "optional category filter",
    "date_range": "optional date context"
  },
  "suggestions": ["Related searches the user might find helpful"]
}

Be helpful in interpreting ambiguous queries."#;

        let scope = request.scope.as_ref()
            .map(|s| s.join(", "))
            .unwrap_or_else(|| "all".to_string());

        let user_prompt = format!("Query: {}\nScope: {}", request.query, scope);

        let context = serde_json::to_value(&request).map_err(|e| AppError::InternalServerError(format!("Failed to serialize context: {}", e)))?;
        let (response, _) = self.call_llm(
            org_id,
            user_id,
            "nl_search",
            system_prompt,
            &user_prompt,
            context,
        ).await?;

        let parsed: serde_json::Value = serde_json::from_str(&response)
            .unwrap_or_else(|_| serde_json::json!({
                "interpreted_query": request.query,
                "keywords": [request.query],
                "suggestions": []
            }));

        let keywords: Vec<String> = parsed["keywords"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_else(|| vec![request.query.clone()]);

        // Perform actual search using keywords
        let mut results = Vec::new();

        // Search controls
        if request.scope.is_none() || request.scope.as_ref().map(|s| s.contains(&"controls".to_string())).unwrap_or(false) {
            let search_pattern = format!("%{}%", keywords.join("%"));
            let controls: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
                r#"
                SELECT id, name, description FROM controls
                WHERE organization_id = $1 AND (LOWER(name) LIKE LOWER($2) OR LOWER(description) LIKE LOWER($2))
                LIMIT 10
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .fetch_all(&self.db)
            .await?;

            for (id, name, desc) in controls {
                results.push(SearchResultItem {
                    entity_type: "control".to_string(),
                    entity_id: id,
                    title: name,
                    description: desc,
                    relevance_score: 0.8,
                    match_reason: "Keyword match in control".to_string(),
                });
            }
        }

        // Search policies
        if request.scope.is_none() || request.scope.as_ref().map(|s| s.contains(&"policies".to_string())).unwrap_or(false) {
            let search_pattern = format!("%{}%", keywords.join("%"));
            let policies: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
                r#"
                SELECT id, title, category FROM policies
                WHERE organization_id = $1 AND (LOWER(title) LIKE LOWER($2) OR LOWER(content) LIKE LOWER($2))
                LIMIT 10
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .fetch_all(&self.db)
            .await?;

            for (id, title, category) in policies {
                results.push(SearchResultItem {
                    entity_type: "policy".to_string(),
                    entity_id: id,
                    title,
                    description: category,
                    relevance_score: 0.8,
                    match_reason: "Keyword match in policy".to_string(),
                });
            }
        }

        // Search risks
        if request.scope.is_none() || request.scope.as_ref().map(|s| s.contains(&"risks".to_string())).unwrap_or(false) {
            let search_pattern = format!("%{}%", keywords.join("%"));
            let risks: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
                r#"
                SELECT id, title, description FROM risks
                WHERE organization_id = $1 AND (LOWER(title) LIKE LOWER($2) OR LOWER(description) LIKE LOWER($2))
                LIMIT 10
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .fetch_all(&self.db)
            .await?;

            for (id, title, desc) in risks {
                results.push(SearchResultItem {
                    entity_type: "risk".to_string(),
                    entity_id: id,
                    title,
                    description: desc,
                    relevance_score: 0.8,
                    match_reason: "Keyword match in risk".to_string(),
                });
            }
        }

        let suggestions = parsed["suggestions"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        Ok(NaturalLanguageSearchResult {
            interpreted_query: parsed["interpreted_query"]
                .as_str()
                .unwrap_or(&request.query)
                .to_string(),
            search_filters: parsed.get("filters").cloned().unwrap_or(serde_json::json!({})),
            results,
            suggestions,
        })
    }

    // ==================== Feature: Audit Preparation ====================

    pub async fn prepare_audit(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        request: AuditPrepRequest,
    ) -> AppResult<AiAuditPreparation> {
        // Get audit details
        let audit: (String, Uuid, Option<String>, Option<String>) = sqlx::query_as(
            r#"
            SELECT a.name, a.framework_id, f.name, a.audit_type
            FROM audits a
            JOIN frameworks f ON a.framework_id = f.id
            WHERE a.id = $1 AND a.organization_id = $2
            "#,
        )
        .bind(request.audit_id)
        .bind(org_id)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AppError::NotFound("Audit not found".into()))?;

        // Get compliance stats for the framework
        let (total_reqs, covered_reqs): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                (SELECT COUNT(*) FROM framework_requirements WHERE framework_id = $1),
                (SELECT COUNT(DISTINCT fr.id) FROM framework_requirements fr
                 JOIN control_requirement_mappings crm ON fr.id = crm.framework_requirement_id
                 JOIN controls c ON crm.control_id = c.id
                 WHERE fr.framework_id = $1 AND c.organization_id = $2 AND c.status = 'implemented')
            "#,
        )
        .bind(audit.1)
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        // Get recent evidence count
        let evidence_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM evidence WHERE organization_id = $1 AND created_at > NOW() - INTERVAL '90 days'"
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        // Get open findings count
        let open_findings: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_findings WHERE audit_id = $1 AND status != 'closed'"
        )
        .bind(request.audit_id)
        .fetch_one(&self.db)
        .await?;

        let system_prompt = r#"You are an audit preparation expert. Create a comprehensive preparation plan for the upcoming compliance audit.

Output a JSON object:
{
  "preparation_summary": "Executive summary of audit readiness and key focus areas",
  "checklist_items": [
    {"item": "Task description", "priority": "high|medium|low", "category": "documentation|evidence|testing|remediation", "estimated_hours": 2}
  ],
  "evidence_gaps": [
    {"area": "Gap area", "description": "What's missing", "recommendation": "How to address"}
  ],
  "risk_areas": [
    {"area": "Risk area", "concern": "Why it's a concern", "mitigation": "Suggested mitigation"}
  ],
  "timeline_suggestions": {
    "immediate": ["Tasks for this week"],
    "short_term": ["Tasks for next 2 weeks"],
    "before_audit": ["Final preparation tasks"]
  }
}

Be specific and actionable. Consider the current compliance posture when making recommendations."#;

        let user_prompt = format!(
            "Audit: {}\nFramework: {}\nAudit Type: {}\n\nCurrent Status:\n- Requirements: {}/{} covered ({:.1}%)\n- Recent Evidence: {} items (last 90 days)\n- Open Findings: {}",
            audit.0,
            audit.2.as_deref().unwrap_or("Unknown"),
            audit.3.as_deref().unwrap_or("Unknown"),
            covered_reqs,
            total_reqs,
            if total_reqs > 0 { (covered_reqs as f64 / total_reqs as f64) * 100.0 } else { 0.0 },
            evidence_count,
            open_findings
        );

        let context = serde_json::json!({
            "audit_id": request.audit_id,
            "framework_id": audit.1,
            "coverage_percent": if total_reqs > 0 { (covered_reqs as f64 / total_reqs as f64) * 100.0 } else { 0.0 }
        });

        let (response, completion_id) = self.call_llm(
            org_id,
            user_id,
            "audit_prep",
            system_prompt,
            &user_prompt,
            context,
        ).await?;

        let parsed: serde_json::Value = serde_json::from_str(&response)
            .unwrap_or_else(|_| serde_json::json!({
                "preparation_summary": response,
                "checklist_items": [],
                "evidence_gaps": [],
                "risk_areas": [],
                "timeline_suggestions": {}
            }));

        let preparation = sqlx::query_as::<_, AiAuditPreparation>(
            r#"
            INSERT INTO ai_audit_preparations
            (audit_id, completion_id, preparation_summary, checklist_items, evidence_gaps, risk_areas, timeline_suggestions)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (audit_id) DO UPDATE SET
                completion_id = EXCLUDED.completion_id,
                preparation_summary = EXCLUDED.preparation_summary,
                checklist_items = EXCLUDED.checklist_items,
                evidence_gaps = EXCLUDED.evidence_gaps,
                risk_areas = EXCLUDED.risk_areas,
                timeline_suggestions = EXCLUDED.timeline_suggestions,
                updated_at = NOW()
            RETURNING id, audit_id, completion_id, preparation_summary, checklist_items, evidence_gaps, risk_areas, timeline_suggestions, created_at, updated_at
            "#,
        )
        .bind(request.audit_id)
        .bind(completion_id)
        .bind(parsed["preparation_summary"].as_str().unwrap_or("Audit preparation analysis complete"))
        .bind(parsed.get("checklist_items").unwrap_or(&serde_json::json!([])))
        .bind(parsed.get("evidence_gaps"))
        .bind(parsed.get("risk_areas"))
        .bind(parsed.get("timeline_suggestions"))
        .fetch_one(&self.db)
        .await?;

        Ok(preparation)
    }

    pub async fn get_audit_preparation(&self, audit_id: Uuid) -> AppResult<Option<AiAuditPreparation>> {
        let prep = sqlx::query_as::<_, AiAuditPreparation>(
            "SELECT * FROM ai_audit_preparations WHERE audit_id = $1"
        )
        .bind(audit_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(prep)
    }

    // ==================== Statistics ====================

    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<AiStats> {
        let config = self.get_configuration(org_id).await?;
        let ai_enabled = config.map(|c| c.enabled).unwrap_or(false);

        let total_completions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ai_completions WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let completions_this_month: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ai_completions WHERE organization_id = $1 AND created_at > DATE_TRUNC('month', NOW())"
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let tokens_used_this_month: i64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(COALESCE(tokens_input, 0) + COALESCE(tokens_output, 0)), 0)
            FROM ai_completions
            WHERE organization_id = $1 AND created_at > DATE_TRUNC('month', NOW())
            "#
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let by_feature: Vec<FeatureUsage> = sqlx::query_as(
            r#"
            SELECT feature, COUNT(*) as count
            FROM ai_completions
            WHERE organization_id = $1
            GROUP BY feature
            ORDER BY count DESC
            "#
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(AiStats {
            total_completions,
            completions_this_month,
            tokens_used_this_month,
            by_feature,
            ai_enabled,
        })
    }
}

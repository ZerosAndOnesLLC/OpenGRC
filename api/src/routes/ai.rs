use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::services::ai::{
    AiAuditPreparation, AiConfiguration, AiEvidenceSummary, AiGapRecommendation,
    AiPolicyDraft, AiRiskAssessment, AiStats, AuditPrepRequest, CreateAiConfiguration,
    EvidenceSummaryRequest, GapAnalysisRequest, NaturalLanguageSearchRequest,
    NaturalLanguageSearchResult, PolicyDraftRequest, RiskScoringRequest,
};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

// ==================== Helpers ====================

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}

fn get_user_id(user: &AuthUser) -> Option<Uuid> {
    Uuid::parse_str(&user.id).ok()
}

// ==================== Configuration ====================

pub async fn get_configuration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Option<AiConfigurationResponse>>> {
    let org_id = get_org_id(&user)?;
    let config = services.ai.get_configuration(org_id).await?;
    Ok(Json(config.map(|c| AiConfigurationResponse {
        id: c.id,
        provider: c.provider,
        api_endpoint: c.api_endpoint,
        model: c.model,
        max_tokens: c.max_tokens,
        temperature: c.temperature.map(|t| t.to_string().parse().unwrap_or(0.7)),
        enabled: c.enabled,
        has_api_key: true,
    })))
}

#[derive(Debug, Serialize)]
pub struct AiConfigurationResponse {
    pub id: Uuid,
    pub provider: String,
    pub api_endpoint: Option<String>,
    pub model: String,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f32>,
    pub enabled: bool,
    pub has_api_key: bool,
}

pub async fn save_configuration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateAiConfiguration>,
) -> AppResult<Json<AiConfiguration>> {
    let org_id = get_org_id(&user)?;
    let config = services.ai.save_configuration(org_id, input).await?;
    Ok(Json(config))
}

#[derive(Debug, Deserialize)]
pub struct ToggleAiRequest {
    pub enabled: bool,
}

pub async fn toggle_ai(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<ToggleAiRequest>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;
    services.ai.toggle_ai(org_id, input.enabled).await?;
    Ok(StatusCode::OK)
}

// ==================== Policy Drafting ====================

pub async fn draft_policy(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(request): Json<PolicyDraftRequest>,
) -> AppResult<Json<AiPolicyDraft>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user);
    let draft = services.ai.draft_policy(org_id, user_id, request).await?;
    Ok(Json(draft))
}

pub async fn list_policy_drafts(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<AiPolicyDraft>>> {
    let org_id = get_org_id(&user)?;
    let drafts = services.ai.list_policy_drafts(org_id).await?;
    Ok(Json(drafts))
}

#[derive(Debug, Deserialize)]
pub struct AcceptDraftRequest {
    pub policy_id: Uuid,
}

pub async fn accept_policy_draft(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(draft_id): Path<Uuid>,
    Json(input): Json<AcceptDraftRequest>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;
    services.ai.accept_policy_draft(org_id, draft_id, input.policy_id).await?;
    Ok(StatusCode::OK)
}

// ==================== Evidence Summarization ====================

pub async fn summarize_evidence(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(request): Json<EvidenceSummaryRequest>,
) -> AppResult<Json<AiEvidenceSummary>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user);
    let summary = services.ai.summarize_evidence(org_id, user_id, request).await?;
    Ok(Json(summary))
}

pub async fn get_evidence_summary(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(evidence_id): Path<Uuid>,
) -> AppResult<Json<Option<AiEvidenceSummary>>> {
    let org_id = get_org_id(&user)?;
    // Verify evidence access
    let _evidence = services.evidence.get_evidence(org_id, evidence_id).await?;

    let summary: Option<AiEvidenceSummary> = sqlx::query_as(
        "SELECT * FROM ai_evidence_summaries WHERE evidence_id = $1"
    )
    .bind(evidence_id)
    .fetch_optional(&services.db)
    .await?;

    Ok(Json(summary))
}

// ==================== Gap Analysis ====================

pub async fn get_gap_recommendations(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(request): Json<GapAnalysisRequest>,
) -> AppResult<Json<Vec<AiGapRecommendation>>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user);
    let recommendations = services.ai.get_gap_recommendations(org_id, user_id, request).await?;
    Ok(Json(recommendations))
}

pub async fn list_gap_recommendations(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(framework_id): Path<Uuid>,
) -> AppResult<Json<Vec<AiGapRecommendation>>> {
    let org_id = get_org_id(&user)?;
    let recommendations: Vec<AiGapRecommendation> = sqlx::query_as(
        r#"
        SELECT * FROM ai_gap_recommendations
        WHERE organization_id = $1 AND framework_id = $2
        ORDER BY created_at DESC
        LIMIT 100
        "#
    )
    .bind(org_id)
    .bind(framework_id)
    .fetch_all(&services.db)
    .await?;

    Ok(Json(recommendations))
}

// ==================== Risk Scoring ====================

pub async fn suggest_risk_scoring(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(request): Json<RiskScoringRequest>,
) -> AppResult<Json<AiRiskAssessment>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user);
    let assessment = services.ai.suggest_risk_scoring(org_id, user_id, request).await?;
    Ok(Json(assessment))
}

pub async fn get_risk_assessment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(risk_id): Path<Uuid>,
) -> AppResult<Json<Option<AiRiskAssessment>>> {
    let org_id = get_org_id(&user)?;
    // Verify risk access
    let _risk = services.risk.get_risk(org_id, risk_id).await?;

    let assessment: Option<AiRiskAssessment> = sqlx::query_as(
        "SELECT * FROM ai_risk_assessments WHERE risk_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(risk_id)
    .fetch_optional(&services.db)
    .await?;

    Ok(Json(assessment))
}

pub async fn accept_risk_assessment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path((risk_id, assessment_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;
    // Verify risk access
    let _risk = services.risk.get_risk(org_id, risk_id).await?;

    services.ai.accept_risk_assessment(assessment_id, risk_id).await?;
    Ok(StatusCode::OK)
}

// ==================== Natural Language Search ====================

pub async fn natural_language_search(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(request): Json<NaturalLanguageSearchRequest>,
) -> AppResult<Json<NaturalLanguageSearchResult>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user);
    let result = services.ai.natural_language_search(org_id, user_id, request).await?;
    Ok(Json(result))
}

// ==================== Audit Preparation ====================

pub async fn prepare_audit(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(request): Json<AuditPrepRequest>,
) -> AppResult<Json<AiAuditPreparation>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user);
    let preparation = services.ai.prepare_audit(org_id, user_id, request).await?;
    Ok(Json(preparation))
}

pub async fn get_audit_preparation(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(audit_id): Path<Uuid>,
) -> AppResult<Json<Option<AiAuditPreparation>>> {
    let org_id = get_org_id(&user)?;
    // Verify audit access
    let _audit = services.audit.get_audit(org_id, audit_id).await?;

    let preparation = services.ai.get_audit_preparation(audit_id).await?;
    Ok(Json(preparation))
}

// ==================== Statistics ====================

pub async fn get_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<AiStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.ai.get_stats(org_id).await?;
    Ok(Json(stats))
}

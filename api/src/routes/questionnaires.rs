use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    CreateQuestionnaireAssignment, CreateQuestionnaireQuestion, CreateQuestionnaireSection,
    CreateQuestionnaireTemplate, ListQuestionnaireAssignmentsQuery, QuestionnaireAssignment,
    QuestionnaireAssignmentWithDetails, QuestionnaireQuestion, QuestionnaireSection,
    QuestionnaireStats, QuestionnaireTemplate, QuestionnaireTemplateWithDetails,
    ReviewQuestionnaireAssignment, SaveQuestionnaireResponse, UpdateQuestionnaireQuestion,
    UpdateQuestionnaireSection, UpdateQuestionnaireTemplate, VendorPortalAccess,
    QuestionnaireResponse,
};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}

fn get_user_id(user: &AuthUser) -> AppResult<Uuid> {
    Uuid::parse_str(&user.id).map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))
}

// ==================== Templates ====================

#[derive(Debug, Deserialize)]
pub struct ListTemplatesParams {
    pub status: Option<String>,
}

pub async fn list_templates(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListTemplatesParams>,
) -> AppResult<Json<Vec<QuestionnaireTemplate>>> {
    let org_id = get_org_id(&user)?;
    let templates = services
        .questionnaire
        .list_templates(org_id, params.status)
        .await?;
    Ok(Json(templates))
}

pub async fn get_template(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<QuestionnaireTemplateWithDetails>> {
    let org_id = get_org_id(&user)?;
    let template = services.questionnaire.get_template(org_id, id).await?;
    Ok(Json(template))
}

pub async fn create_template(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateQuestionnaireTemplate>,
) -> AppResult<Json<QuestionnaireTemplate>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let template = services
        .questionnaire
        .create_template(org_id, Some(user_id), input)
        .await?;
    Ok(Json(template))
}

pub async fn update_template(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateQuestionnaireTemplate>,
) -> AppResult<Json<QuestionnaireTemplate>> {
    let org_id = get_org_id(&user)?;
    let template = services
        .questionnaire
        .update_template(org_id, id, input)
        .await?;
    Ok(Json(template))
}

pub async fn delete_template(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services.questionnaire.delete_template(org_id, id).await?;
    Ok(Json(()))
}

pub async fn publish_template(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<QuestionnaireTemplate>> {
    let org_id = get_org_id(&user)?;
    let template = services.questionnaire.publish_template(org_id, id).await?;
    Ok(Json(template))
}

// ==================== Sections ====================

#[derive(Debug, Deserialize)]
pub struct SectionPath {
    pub template_id: Uuid,
    pub section_id: Uuid,
}

pub async fn create_section(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(template_id): Path<Uuid>,
    Json(input): Json<CreateQuestionnaireSection>,
) -> AppResult<Json<QuestionnaireSection>> {
    let org_id = get_org_id(&user)?;
    let section = services
        .questionnaire
        .create_section(org_id, template_id, input)
        .await?;
    Ok(Json(section))
}

pub async fn update_section(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<SectionPath>,
    Json(input): Json<UpdateQuestionnaireSection>,
) -> AppResult<Json<QuestionnaireSection>> {
    let org_id = get_org_id(&user)?;
    let section = services
        .questionnaire
        .update_section(org_id, path.template_id, path.section_id, input)
        .await?;
    Ok(Json(section))
}

pub async fn delete_section(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<SectionPath>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services
        .questionnaire
        .delete_section(org_id, path.template_id, path.section_id)
        .await?;
    Ok(Json(()))
}

// ==================== Questions ====================

#[derive(Debug, Deserialize)]
pub struct QuestionPath {
    pub template_id: Uuid,
    pub question_id: Uuid,
}

pub async fn create_question(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(template_id): Path<Uuid>,
    Json(input): Json<CreateQuestionnaireQuestion>,
) -> AppResult<Json<QuestionnaireQuestion>> {
    let org_id = get_org_id(&user)?;
    let question = services
        .questionnaire
        .create_question(org_id, template_id, input)
        .await?;
    Ok(Json(question))
}

pub async fn update_question(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<QuestionPath>,
    Json(input): Json<UpdateQuestionnaireQuestion>,
) -> AppResult<Json<QuestionnaireQuestion>> {
    let org_id = get_org_id(&user)?;
    let question = services
        .questionnaire
        .update_question(org_id, path.template_id, path.question_id, input)
        .await?;
    Ok(Json(question))
}

pub async fn delete_question(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<QuestionPath>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services
        .questionnaire
        .delete_question(org_id, path.template_id, path.question_id)
        .await?;
    Ok(Json(()))
}

// ==================== Assignments ====================

#[derive(Debug, Deserialize)]
pub struct ListAssignmentsParams {
    pub vendor_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListAssignmentsParams> for ListQuestionnaireAssignmentsQuery {
    fn from(params: ListAssignmentsParams) -> Self {
        ListQuestionnaireAssignmentsQuery {
            vendor_id: params.vendor_id,
            status: params.status,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

pub async fn list_assignments(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListAssignmentsParams>,
) -> AppResult<Json<Vec<QuestionnaireAssignmentWithDetails>>> {
    let org_id = get_org_id(&user)?;
    let assignments = services
        .questionnaire
        .list_assignments(org_id, params.into())
        .await?;
    Ok(Json(assignments))
}

pub async fn get_assignment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<QuestionnaireAssignmentWithDetails>> {
    let org_id = get_org_id(&user)?;
    let assignment = services.questionnaire.get_assignment(org_id, id).await?;
    Ok(Json(assignment))
}

pub async fn create_assignment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateQuestionnaireAssignment>,
) -> AppResult<Json<QuestionnaireAssignment>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let assignment = services
        .questionnaire
        .create_assignment(org_id, Some(user_id), input)
        .await?;
    Ok(Json(assignment))
}

pub async fn review_assignment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<ReviewQuestionnaireAssignment>,
) -> AppResult<Json<QuestionnaireAssignment>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let assignment = services
        .questionnaire
        .review_assignment(org_id, id, Some(user_id), input)
        .await?;
    Ok(Json(assignment))
}

pub async fn delete_assignment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services.questionnaire.delete_assignment(org_id, id).await?;
    Ok(Json(()))
}

// ==================== Statistics ====================

pub async fn get_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<QuestionnaireStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.questionnaire.get_stats(org_id).await?;
    Ok(Json(stats))
}

// ==================== Vendor Portal (Public) ====================

#[derive(Debug, Deserialize)]
pub struct PortalAccessQuery {
    pub token: String,
}

pub async fn get_portal_access(
    State(services): State<Arc<AppServices>>,
    Query(query): Query<PortalAccessQuery>,
) -> AppResult<Json<VendorPortalAccess>> {
    let access = services.questionnaire.get_portal_access(&query.token).await?;
    Ok(Json(access))
}

pub async fn save_portal_response(
    State(services): State<Arc<AppServices>>,
    Query(query): Query<PortalAccessQuery>,
    Json(input): Json<SaveQuestionnaireResponse>,
) -> AppResult<Json<QuestionnaireResponse>> {
    let response = services
        .questionnaire
        .save_response(&query.token, input)
        .await?;
    Ok(Json(response))
}

pub async fn submit_portal_questionnaire(
    State(services): State<Arc<AppServices>>,
    Query(query): Query<PortalAccessQuery>,
) -> AppResult<Json<()>> {
    services
        .questionnaire
        .submit_questionnaire(&query.token)
        .await?;
    Ok(Json(()))
}

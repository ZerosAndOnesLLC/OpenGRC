use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ==================== TEMPLATES ====================

/// Questionnaire template
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestionnaireTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_default: bool,
    pub version: i32,
    pub status: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Template with sections and questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireTemplateWithDetails {
    #[serde(flatten)]
    pub template: QuestionnaireTemplate,
    pub sections: Vec<QuestionnaireSectionWithQuestions>,
    pub question_count: i64,
}

/// Create template request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuestionnaireTemplate {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_default: Option<bool>,
}

/// Update template request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQuestionnaireTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_default: Option<bool>,
    pub status: Option<String>,
}

// ==================== SECTIONS ====================

/// Questionnaire section
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestionnaireSection {
    pub id: Uuid,
    pub template_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

/// Section with questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireSectionWithQuestions {
    #[serde(flatten)]
    pub section: QuestionnaireSection,
    pub questions: Vec<QuestionnaireQuestion>,
}

/// Create section request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuestionnaireSection {
    pub name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

/// Update section request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQuestionnaireSection {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

// ==================== QUESTIONS ====================

/// Questionnaire question
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestionnaireQuestion {
    pub id: Uuid,
    pub template_id: Uuid,
    pub section_id: Option<Uuid>,
    pub question_text: String,
    pub help_text: Option<String>,
    pub question_type: String,
    pub options: Option<serde_json::Value>,
    pub is_required: bool,
    pub weight: i32,
    pub risk_mapping: Option<String>,
    pub control_codes: Option<Vec<String>>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create question request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuestionnaireQuestion {
    pub section_id: Option<Uuid>,
    pub question_text: String,
    pub help_text: Option<String>,
    pub question_type: String, // text, textarea, single_choice, multiple_choice, yes_no, file_upload, date, number
    pub options: Option<serde_json::Value>,
    pub is_required: Option<bool>,
    pub weight: Option<i32>,
    pub risk_mapping: Option<String>,
    pub control_codes: Option<Vec<String>>,
    pub sort_order: Option<i32>,
}

/// Update question request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQuestionnaireQuestion {
    pub section_id: Option<Uuid>,
    pub question_text: Option<String>,
    pub help_text: Option<String>,
    pub question_type: Option<String>,
    pub options: Option<serde_json::Value>,
    pub is_required: Option<bool>,
    pub weight: Option<i32>,
    pub risk_mapping: Option<String>,
    pub control_codes: Option<Vec<String>>,
    pub sort_order: Option<i32>,
}

// ==================== ASSIGNMENTS ====================

/// Questionnaire assignment (sent to vendor)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestionnaireAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub vendor_id: Uuid,
    pub access_token: String,
    pub status: Option<String>,
    pub assigned_by: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
    pub due_date: Option<NaiveDate>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
    pub score: Option<Decimal>,
    pub risk_rating: Option<String>,
    pub reminder_sent_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Assignment with vendor and template info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireAssignmentWithDetails {
    #[serde(flatten)]
    pub assignment: QuestionnaireAssignment,
    pub vendor_name: String,
    pub template_name: String,
    pub response_count: i64,
    pub question_count: i64,
}

/// Create assignment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuestionnaireAssignment {
    pub template_id: Uuid,
    pub vendor_id: Uuid,
    pub due_date: Option<NaiveDate>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Review assignment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewQuestionnaireAssignment {
    pub status: String, // approved, rejected
    pub review_notes: Option<String>,
}

/// List assignments query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListQuestionnaireAssignmentsQuery {
    pub vendor_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ==================== RESPONSES ====================

/// Questionnaire response
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestionnaireResponse {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub question_id: Uuid,
    pub response_text: Option<String>,
    pub response_value: Option<serde_json::Value>,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub answered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response with question info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireResponseWithQuestion {
    #[serde(flatten)]
    pub response: QuestionnaireResponse,
    pub question_text: String,
    pub question_type: String,
    pub is_required: bool,
}

/// Save response request (from vendor portal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveQuestionnaireResponse {
    pub question_id: Uuid,
    pub response_text: Option<String>,
    pub response_value: Option<serde_json::Value>,
}

/// Bulk save responses request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkSaveQuestionnaireResponses {
    pub responses: Vec<SaveQuestionnaireResponse>,
}

// ==================== RESPONSE COMMENTS ====================

/// Comment on a response
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestionnaireResponseComment {
    pub id: Uuid,
    pub response_id: Uuid,
    pub user_id: Uuid,
    pub comment: String,
    pub is_internal: bool,
    pub created_at: DateTime<Utc>,
}

/// Create comment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponseComment {
    pub comment: String,
    pub is_internal: Option<bool>,
}

// ==================== VENDOR PORTAL ====================

/// Vendor portal access (for external vendors to fill questionnaires)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorPortalAccess {
    pub assignment_id: Uuid,
    pub vendor_name: String,
    pub organization_name: String,
    pub template_name: String,
    pub due_date: Option<NaiveDate>,
    pub status: String,
    pub sections: Vec<QuestionnaireSectionWithQuestions>,
    pub responses: Vec<QuestionnaireResponse>,
}

/// Vendor portal submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorPortalSubmission {
    pub responses: Vec<SaveQuestionnaireResponse>,
}

// ==================== STATISTICS ====================

/// Questionnaire statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionnaireStats {
    pub total_templates: i64,
    pub published_templates: i64,
    pub total_assignments: i64,
    pub pending_assignments: i64,
    pub submitted_assignments: i64,
    pub overdue_assignments: i64,
    pub average_completion_rate: f64,
}

// ==================== VALIDATION ====================

impl QuestionnaireTemplate {
    pub fn validate_create(input: &CreateQuestionnaireTemplate) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return Err("Template name is required".to_string());
        }
        if input.name.len() > 255 {
            return Err("Template name must be 255 characters or less".to_string());
        }
        Ok(())
    }
}

impl QuestionnaireQuestion {
    pub fn validate_create(input: &CreateQuestionnaireQuestion) -> Result<(), String> {
        if input.question_text.trim().is_empty() {
            return Err("Question text is required".to_string());
        }

        let valid_types = [
            "text",
            "textarea",
            "single_choice",
            "multiple_choice",
            "yes_no",
            "file_upload",
            "date",
            "number",
        ];
        if !valid_types.contains(&input.question_type.as_str()) {
            return Err(format!(
                "Invalid question type. Must be one of: {}",
                valid_types.join(", ")
            ));
        }

        // Validate options for choice questions
        if matches!(
            input.question_type.as_str(),
            "single_choice" | "multiple_choice"
        ) {
            if input.options.is_none() {
                return Err("Options are required for choice questions".to_string());
            }
        }

        Ok(())
    }
}

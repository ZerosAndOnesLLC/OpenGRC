use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    CreateQuestionnaireAssignment, CreateQuestionnaireQuestion, CreateQuestionnaireSection,
    CreateQuestionnaireTemplate, ListQuestionnaireAssignmentsQuery, QuestionnaireAssignment,
    QuestionnaireAssignmentWithDetails, QuestionnaireQuestion, QuestionnaireResponse,
    QuestionnaireSection, QuestionnaireSectionWithQuestions, QuestionnaireStats,
    QuestionnaireTemplate, QuestionnaireTemplateWithDetails, ReviewQuestionnaireAssignment,
    SaveQuestionnaireResponse, UpdateQuestionnaireQuestion, UpdateQuestionnaireSection,
    UpdateQuestionnaireTemplate, VendorPortalAccess,
};
use crate::utils::{AppError, AppResult};
use rand::Rng;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(1800);
const CACHE_PREFIX_TEMPLATE: &str = "questionnaire:template";
const CACHE_PREFIX_STATS: &str = "questionnaire:stats";

#[derive(Clone)]
pub struct QuestionnaireService {
    db: PgPool,
    cache: CacheClient,
}

impl QuestionnaireService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Templates ====================

    /// List questionnaire templates
    pub async fn list_templates(
        &self,
        org_id: Uuid,
        status: Option<String>,
    ) -> AppResult<Vec<QuestionnaireTemplate>> {
        let templates = sqlx::query_as::<_, QuestionnaireTemplate>(
            r#"
            SELECT id, organization_id, name, description, category, is_default,
                   version, status, created_by, created_at, updated_at
            FROM questionnaire_templates
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
            ORDER BY name ASC
            "#,
        )
        .bind(org_id)
        .bind(&status)
        .fetch_all(&self.db)
        .await?;

        Ok(templates)
    }

    /// Get a template with all sections and questions
    pub async fn get_template(
        &self,
        org_id: Uuid,
        template_id: Uuid,
    ) -> AppResult<QuestionnaireTemplateWithDetails> {
        let cache_key = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_TEMPLATE,
            &template_id.to_string(),
        );

        if let Some(cached) = self
            .cache
            .get::<QuestionnaireTemplateWithDetails>(&cache_key)
            .await?
        {
            return Ok(cached);
        }

        let template = sqlx::query_as::<_, QuestionnaireTemplate>(
            r#"
            SELECT id, organization_id, name, description, category, is_default,
                   version, status, created_by, created_at, updated_at
            FROM questionnaire_templates
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(template_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Template {} not found", template_id)))?;

        let sections = self.get_sections_with_questions(template_id).await?;

        let (question_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM questionnaire_questions WHERE template_id = $1",
        )
        .bind(template_id)
        .fetch_one(&self.db)
        .await?;

        let result = QuestionnaireTemplateWithDetails {
            template,
            sections,
            question_count,
        };

        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create a template
    pub async fn create_template(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        input: CreateQuestionnaireTemplate,
    ) -> AppResult<QuestionnaireTemplate> {
        QuestionnaireTemplate::validate_create(&input).map_err(AppError::ValidationError)?;

        let template = sqlx::query_as::<_, QuestionnaireTemplate>(
            r#"
            INSERT INTO questionnaire_templates (organization_id, name, description, category, is_default, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, organization_id, name, description, category, is_default,
                      version, status, created_by, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.category)
        .bind(input.is_default.unwrap_or(false))
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_stats_cache(org_id).await?;

        tracing::info!("Created questionnaire template: {} ({})", template.name, template.id);

        Ok(template)
    }

    /// Update a template
    pub async fn update_template(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        input: UpdateQuestionnaireTemplate,
    ) -> AppResult<QuestionnaireTemplate> {
        // Verify exists
        self.get_template(org_id, template_id).await?;

        let template = sqlx::query_as::<_, QuestionnaireTemplate>(
            r#"
            UPDATE questionnaire_templates
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                category = COALESCE($5, category),
                is_default = COALESCE($6, is_default),
                status = COALESCE($7, status),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, name, description, category, is_default,
                      version, status, created_by, created_at, updated_at
            "#,
        )
        .bind(template_id)
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.category)
        .bind(input.is_default)
        .bind(&input.status)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_template_cache(org_id, template_id).await?;

        tracing::info!("Updated questionnaire template: {}", template_id);

        Ok(template)
    }

    /// Delete a template
    pub async fn delete_template(&self, org_id: Uuid, template_id: Uuid) -> AppResult<()> {
        self.get_template(org_id, template_id).await?;

        sqlx::query("DELETE FROM questionnaire_templates WHERE id = $1 AND organization_id = $2")
            .bind(template_id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        self.invalidate_template_cache(org_id, template_id).await?;

        tracing::info!("Deleted questionnaire template: {}", template_id);

        Ok(())
    }

    /// Publish a template
    pub async fn publish_template(
        &self,
        org_id: Uuid,
        template_id: Uuid,
    ) -> AppResult<QuestionnaireTemplate> {
        self.update_template(
            org_id,
            template_id,
            UpdateQuestionnaireTemplate {
                name: None,
                description: None,
                category: None,
                is_default: None,
                status: Some("published".to_string()),
            },
        )
        .await
    }

    // ==================== Sections ====================

    async fn get_sections_with_questions(
        &self,
        template_id: Uuid,
    ) -> AppResult<Vec<QuestionnaireSectionWithQuestions>> {
        let sections = sqlx::query_as::<_, QuestionnaireSection>(
            r#"
            SELECT id, template_id, name, description, sort_order, created_at
            FROM questionnaire_sections
            WHERE template_id = $1
            ORDER BY sort_order ASC, created_at ASC
            "#,
        )
        .bind(template_id)
        .fetch_all(&self.db)
        .await?;

        let questions = sqlx::query_as::<_, QuestionnaireQuestion>(
            r#"
            SELECT id, template_id, section_id, question_text, help_text, question_type,
                   options, is_required, weight, risk_mapping, control_codes, sort_order,
                   created_at, updated_at
            FROM questionnaire_questions
            WHERE template_id = $1
            ORDER BY sort_order ASC, created_at ASC
            "#,
        )
        .bind(template_id)
        .fetch_all(&self.db)
        .await?;

        let mut result = Vec::new();

        // Questions without section go into a default section
        let unsectioned_questions: Vec<QuestionnaireQuestion> = questions
            .iter()
            .filter(|q| q.section_id.is_none())
            .cloned()
            .collect();

        if !unsectioned_questions.is_empty() {
            result.push(QuestionnaireSectionWithQuestions {
                section: QuestionnaireSection {
                    id: Uuid::nil(),
                    template_id,
                    name: "General".to_string(),
                    description: None,
                    sort_order: -1,
                    created_at: chrono::Utc::now(),
                },
                questions: unsectioned_questions,
            });
        }

        for section in sections {
            let section_questions: Vec<QuestionnaireQuestion> = questions
                .iter()
                .filter(|q| q.section_id == Some(section.id))
                .cloned()
                .collect();

            result.push(QuestionnaireSectionWithQuestions {
                section,
                questions: section_questions,
            });
        }

        Ok(result)
    }

    /// Create a section
    pub async fn create_section(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        input: CreateQuestionnaireSection,
    ) -> AppResult<QuestionnaireSection> {
        // Verify template exists
        self.get_template(org_id, template_id).await?;

        let sort_order = input.sort_order.unwrap_or(0);

        let section = sqlx::query_as::<_, QuestionnaireSection>(
            r#"
            INSERT INTO questionnaire_sections (template_id, name, description, sort_order)
            VALUES ($1, $2, $3, $4)
            RETURNING id, template_id, name, description, sort_order, created_at
            "#,
        )
        .bind(template_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(sort_order)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_template_cache(org_id, template_id).await?;

        tracing::info!("Created questionnaire section: {}", section.id);

        Ok(section)
    }

    /// Update a section
    pub async fn update_section(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        section_id: Uuid,
        input: UpdateQuestionnaireSection,
    ) -> AppResult<QuestionnaireSection> {
        // Verify template exists
        self.get_template(org_id, template_id).await?;

        let section = sqlx::query_as::<_, QuestionnaireSection>(
            r#"
            UPDATE questionnaire_sections
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                sort_order = COALESCE($5, sort_order)
            WHERE id = $1 AND template_id = $2
            RETURNING id, template_id, name, description, sort_order, created_at
            "#,
        )
        .bind(section_id)
        .bind(template_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.sort_order)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_template_cache(org_id, template_id).await?;

        tracing::info!("Updated questionnaire section: {}", section_id);

        Ok(section)
    }

    /// Delete a section
    pub async fn delete_section(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        section_id: Uuid,
    ) -> AppResult<()> {
        // Verify template exists
        self.get_template(org_id, template_id).await?;

        // Move questions to no section
        sqlx::query(
            "UPDATE questionnaire_questions SET section_id = NULL WHERE section_id = $1",
        )
        .bind(section_id)
        .execute(&self.db)
        .await?;

        sqlx::query("DELETE FROM questionnaire_sections WHERE id = $1 AND template_id = $2")
            .bind(section_id)
            .bind(template_id)
            .execute(&self.db)
            .await?;

        self.invalidate_template_cache(org_id, template_id).await?;

        tracing::info!("Deleted questionnaire section: {}", section_id);

        Ok(())
    }

    // ==================== Questions ====================

    /// Create a question
    pub async fn create_question(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        input: CreateQuestionnaireQuestion,
    ) -> AppResult<QuestionnaireQuestion> {
        // Verify template exists
        self.get_template(org_id, template_id).await?;

        QuestionnaireQuestion::validate_create(&input).map_err(AppError::ValidationError)?;

        let sort_order = input.sort_order.unwrap_or(0);
        let weight = input.weight.unwrap_or(1);
        let is_required = input.is_required.unwrap_or(true);

        let question = sqlx::query_as::<_, QuestionnaireQuestion>(
            r#"
            INSERT INTO questionnaire_questions
                (template_id, section_id, question_text, help_text, question_type,
                 options, is_required, weight, risk_mapping, control_codes, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, template_id, section_id, question_text, help_text, question_type,
                      options, is_required, weight, risk_mapping, control_codes, sort_order,
                      created_at, updated_at
            "#,
        )
        .bind(template_id)
        .bind(input.section_id)
        .bind(&input.question_text)
        .bind(&input.help_text)
        .bind(&input.question_type)
        .bind(&input.options)
        .bind(is_required)
        .bind(weight)
        .bind(&input.risk_mapping)
        .bind(&input.control_codes)
        .bind(sort_order)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_template_cache(org_id, template_id).await?;

        tracing::info!("Created questionnaire question: {}", question.id);

        Ok(question)
    }

    /// Update a question
    pub async fn update_question(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        question_id: Uuid,
        input: UpdateQuestionnaireQuestion,
    ) -> AppResult<QuestionnaireQuestion> {
        // Verify template exists
        self.get_template(org_id, template_id).await?;

        let question = sqlx::query_as::<_, QuestionnaireQuestion>(
            r#"
            UPDATE questionnaire_questions
            SET
                section_id = COALESCE($3, section_id),
                question_text = COALESCE($4, question_text),
                help_text = COALESCE($5, help_text),
                question_type = COALESCE($6, question_type),
                options = COALESCE($7, options),
                is_required = COALESCE($8, is_required),
                weight = COALESCE($9, weight),
                risk_mapping = COALESCE($10, risk_mapping),
                control_codes = COALESCE($11, control_codes),
                sort_order = COALESCE($12, sort_order),
                updated_at = NOW()
            WHERE id = $1 AND template_id = $2
            RETURNING id, template_id, section_id, question_text, help_text, question_type,
                      options, is_required, weight, risk_mapping, control_codes, sort_order,
                      created_at, updated_at
            "#,
        )
        .bind(question_id)
        .bind(template_id)
        .bind(input.section_id)
        .bind(&input.question_text)
        .bind(&input.help_text)
        .bind(&input.question_type)
        .bind(&input.options)
        .bind(input.is_required)
        .bind(input.weight)
        .bind(&input.risk_mapping)
        .bind(&input.control_codes)
        .bind(input.sort_order)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_template_cache(org_id, template_id).await?;

        tracing::info!("Updated questionnaire question: {}", question_id);

        Ok(question)
    }

    /// Delete a question
    pub async fn delete_question(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        question_id: Uuid,
    ) -> AppResult<()> {
        // Verify template exists
        self.get_template(org_id, template_id).await?;

        sqlx::query("DELETE FROM questionnaire_questions WHERE id = $1 AND template_id = $2")
            .bind(question_id)
            .bind(template_id)
            .execute(&self.db)
            .await?;

        self.invalidate_template_cache(org_id, template_id).await?;

        tracing::info!("Deleted questionnaire question: {}", question_id);

        Ok(())
    }

    // ==================== Assignments ====================

    /// List assignments
    pub async fn list_assignments(
        &self,
        org_id: Uuid,
        query: ListQuestionnaireAssignmentsQuery,
    ) -> AppResult<Vec<QuestionnaireAssignmentWithDetails>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        let assignments = sqlx::query_as::<_, QuestionnaireAssignment>(
            r#"
            SELECT qa.id, qa.organization_id, qa.template_id, qa.vendor_id, qa.access_token,
                   qa.status, qa.assigned_by, qa.assigned_at, qa.due_date, qa.submitted_at,
                   qa.reviewed_by, qa.reviewed_at, qa.review_notes, qa.score, qa.risk_rating,
                   qa.reminder_sent_at, qa.expires_at, qa.created_at, qa.updated_at
            FROM questionnaire_assignments qa
            WHERE qa.organization_id = $1
              AND ($2::uuid IS NULL OR qa.vendor_id = $2)
              AND ($3::text IS NULL OR qa.status = $3)
            ORDER BY qa.created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(org_id)
        .bind(query.vendor_id)
        .bind(&query.status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let mut result = Vec::new();

        for assignment in assignments {
            let details = self.get_assignment_details(&assignment).await?;
            result.push(details);
        }

        Ok(result)
    }

    /// Get assignment details
    async fn get_assignment_details(
        &self,
        assignment: &QuestionnaireAssignment,
    ) -> AppResult<QuestionnaireAssignmentWithDetails> {
        let (vendor_name,): (String,) =
            sqlx::query_as("SELECT name FROM vendors WHERE id = $1")
                .bind(assignment.vendor_id)
                .fetch_one(&self.db)
                .await?;

        let (template_name,): (String,) =
            sqlx::query_as("SELECT name FROM questionnaire_templates WHERE id = $1")
                .bind(assignment.template_id)
                .fetch_one(&self.db)
                .await?;

        let (response_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM questionnaire_responses WHERE assignment_id = $1 AND response_text IS NOT NULL",
        )
        .bind(assignment.id)
        .fetch_one(&self.db)
        .await?;

        let (question_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM questionnaire_questions WHERE template_id = $1",
        )
        .bind(assignment.template_id)
        .fetch_one(&self.db)
        .await?;

        Ok(QuestionnaireAssignmentWithDetails {
            assignment: assignment.clone(),
            vendor_name,
            template_name,
            response_count,
            question_count,
        })
    }

    /// Get a single assignment
    pub async fn get_assignment(
        &self,
        org_id: Uuid,
        assignment_id: Uuid,
    ) -> AppResult<QuestionnaireAssignmentWithDetails> {
        let assignment = sqlx::query_as::<_, QuestionnaireAssignment>(
            r#"
            SELECT id, organization_id, template_id, vendor_id, access_token,
                   status, assigned_by, assigned_at, due_date, submitted_at,
                   reviewed_by, reviewed_at, review_notes, score, risk_rating,
                   reminder_sent_at, expires_at, created_at, updated_at
            FROM questionnaire_assignments
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(assignment_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Assignment {} not found", assignment_id)))?;

        self.get_assignment_details(&assignment).await
    }

    /// Create an assignment (send questionnaire to vendor)
    pub async fn create_assignment(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        input: CreateQuestionnaireAssignment,
    ) -> AppResult<QuestionnaireAssignment> {
        // Generate secure access token
        let access_token = generate_access_token();

        // Default expiration: 30 days from now
        let expires_at = input.expires_at.unwrap_or_else(|| {
            chrono::Utc::now() + chrono::Duration::days(30)
        });

        let assignment = sqlx::query_as::<_, QuestionnaireAssignment>(
            r#"
            INSERT INTO questionnaire_assignments
                (organization_id, template_id, vendor_id, access_token, assigned_by, due_date, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, organization_id, template_id, vendor_id, access_token,
                      status, assigned_by, assigned_at, due_date, submitted_at,
                      reviewed_by, reviewed_at, review_notes, score, risk_rating,
                      reminder_sent_at, expires_at, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(input.template_id)
        .bind(input.vendor_id)
        .bind(&access_token)
        .bind(user_id)
        .bind(input.due_date)
        .bind(expires_at)
        .fetch_one(&self.db)
        .await?;

        // Create empty responses for all questions
        sqlx::query(
            r#"
            INSERT INTO questionnaire_responses (assignment_id, question_id)
            SELECT $1, id FROM questionnaire_questions WHERE template_id = $2
            "#,
        )
        .bind(assignment.id)
        .bind(input.template_id)
        .execute(&self.db)
        .await?;

        self.invalidate_stats_cache(org_id).await?;

        tracing::info!("Created questionnaire assignment: {}", assignment.id);

        Ok(assignment)
    }

    /// Review an assignment
    pub async fn review_assignment(
        &self,
        org_id: Uuid,
        assignment_id: Uuid,
        user_id: Option<Uuid>,
        input: ReviewQuestionnaireAssignment,
    ) -> AppResult<QuestionnaireAssignment> {
        let valid_statuses = ["approved", "rejected"];
        if !valid_statuses.contains(&input.status.as_str()) {
            return Err(AppError::ValidationError(
                "Status must be 'approved' or 'rejected'".to_string(),
            ));
        }

        let assignment = sqlx::query_as::<_, QuestionnaireAssignment>(
            r#"
            UPDATE questionnaire_assignments
            SET
                status = $3,
                reviewed_by = $4,
                reviewed_at = NOW(),
                review_notes = $5,
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, template_id, vendor_id, access_token,
                      status, assigned_by, assigned_at, due_date, submitted_at,
                      reviewed_by, reviewed_at, review_notes, score, risk_rating,
                      reminder_sent_at, expires_at, created_at, updated_at
            "#,
        )
        .bind(assignment_id)
        .bind(org_id)
        .bind(&input.status)
        .bind(user_id)
        .bind(&input.review_notes)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_stats_cache(org_id).await?;

        tracing::info!("Reviewed questionnaire assignment: {} -> {}", assignment_id, input.status);

        Ok(assignment)
    }

    /// Delete an assignment
    pub async fn delete_assignment(&self, org_id: Uuid, assignment_id: Uuid) -> AppResult<()> {
        self.get_assignment(org_id, assignment_id).await?;

        sqlx::query(
            "DELETE FROM questionnaire_assignments WHERE id = $1 AND organization_id = $2",
        )
        .bind(assignment_id)
        .bind(org_id)
        .execute(&self.db)
        .await?;

        self.invalidate_stats_cache(org_id).await?;

        tracing::info!("Deleted questionnaire assignment: {}", assignment_id);

        Ok(())
    }

    // ==================== Vendor Portal ====================

    /// Get vendor portal access (by token)
    pub async fn get_portal_access(&self, access_token: &str) -> AppResult<VendorPortalAccess> {
        let assignment = sqlx::query_as::<_, QuestionnaireAssignment>(
            r#"
            SELECT id, organization_id, template_id, vendor_id, access_token,
                   status, assigned_by, assigned_at, due_date, submitted_at,
                   reviewed_by, reviewed_at, review_notes, score, risk_rating,
                   reminder_sent_at, expires_at, created_at, updated_at
            FROM questionnaire_assignments
            WHERE access_token = $1
            "#,
        )
        .bind(access_token)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Invalid or expired access token".to_string()))?;

        // Check if expired
        if let Some(expires_at) = assignment.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(AppError::BadRequest("This questionnaire link has expired".to_string()));
            }
        }

        // Check if already submitted and reviewed
        if matches!(assignment.status.as_deref(), Some("approved") | Some("rejected")) {
            return Err(AppError::BadRequest(
                "This questionnaire has already been reviewed".to_string(),
            ));
        }

        let (vendor_name,): (String,) =
            sqlx::query_as("SELECT name FROM vendors WHERE id = $1")
                .bind(assignment.vendor_id)
                .fetch_one(&self.db)
                .await?;

        let (organization_name,): (String,) =
            sqlx::query_as("SELECT name FROM organizations WHERE id = $1")
                .bind(assignment.organization_id)
                .fetch_one(&self.db)
                .await?;

        let (template_name,): (String,) =
            sqlx::query_as("SELECT name FROM questionnaire_templates WHERE id = $1")
                .bind(assignment.template_id)
                .fetch_one(&self.db)
                .await?;

        let sections = self
            .get_sections_with_questions(assignment.template_id)
            .await?;

        let responses = sqlx::query_as::<_, QuestionnaireResponse>(
            r#"
            SELECT id, assignment_id, question_id, response_text, response_value,
                   file_path, file_name, answered_at, created_at, updated_at
            FROM questionnaire_responses
            WHERE assignment_id = $1
            "#,
        )
        .bind(assignment.id)
        .fetch_all(&self.db)
        .await?;

        Ok(VendorPortalAccess {
            assignment_id: assignment.id,
            vendor_name,
            organization_name,
            template_name,
            due_date: assignment.due_date,
            status: assignment.status.unwrap_or_else(|| "pending".to_string()),
            sections,
            responses,
        })
    }

    /// Save a response (from vendor portal)
    pub async fn save_response(
        &self,
        access_token: &str,
        input: SaveQuestionnaireResponse,
    ) -> AppResult<QuestionnaireResponse> {
        // Verify access
        let portal = self.get_portal_access(access_token).await?;

        if matches!(portal.status.as_str(), "submitted" | "approved" | "rejected") {
            return Err(AppError::BadRequest(
                "Cannot modify responses after submission".to_string(),
            ));
        }

        let response = sqlx::query_as::<_, QuestionnaireResponse>(
            r#"
            UPDATE questionnaire_responses
            SET
                response_text = $3,
                response_value = $4,
                answered_at = NOW(),
                updated_at = NOW()
            WHERE assignment_id = $1 AND question_id = $2
            RETURNING id, assignment_id, question_id, response_text, response_value,
                      file_path, file_name, answered_at, created_at, updated_at
            "#,
        )
        .bind(portal.assignment_id)
        .bind(input.question_id)
        .bind(&input.response_text)
        .bind(&input.response_value)
        .fetch_one(&self.db)
        .await?;

        // Update assignment status to in_progress
        sqlx::query(
            "UPDATE questionnaire_assignments SET status = 'in_progress', updated_at = NOW() WHERE id = $1 AND status = 'pending'",
        )
        .bind(portal.assignment_id)
        .execute(&self.db)
        .await?;

        tracing::info!(
            "Saved questionnaire response for assignment {}, question {}",
            portal.assignment_id,
            input.question_id
        );

        Ok(response)
    }

    /// Submit questionnaire (from vendor portal)
    pub async fn submit_questionnaire(&self, access_token: &str) -> AppResult<()> {
        let portal = self.get_portal_access(access_token).await?;

        if portal.status == "submitted" {
            return Err(AppError::BadRequest(
                "Questionnaire has already been submitted".to_string(),
            ));
        }

        // Check required questions are answered
        let (unanswered_required,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM questionnaire_questions q
            JOIN questionnaire_responses r ON q.id = r.question_id
            WHERE r.assignment_id = $1
              AND q.is_required = true
              AND r.response_text IS NULL
              AND r.response_value IS NULL
            "#,
        )
        .bind(portal.assignment_id)
        .fetch_one(&self.db)
        .await?;

        if unanswered_required > 0 {
            return Err(AppError::ValidationError(format!(
                "{} required questions have not been answered",
                unanswered_required
            )));
        }

        // Calculate score
        let score = self.calculate_score(portal.assignment_id).await?;
        let risk_rating = calculate_risk_rating(score);

        sqlx::query(
            r#"
            UPDATE questionnaire_assignments
            SET status = 'submitted', submitted_at = NOW(), score = $2, risk_rating = $3, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(portal.assignment_id)
        .bind(score)
        .bind(&risk_rating)
        .execute(&self.db)
        .await?;

        tracing::info!(
            "Submitted questionnaire: {} (score: {}, risk: {})",
            portal.assignment_id,
            score,
            risk_rating
        );

        Ok(())
    }

    /// Calculate questionnaire score
    async fn calculate_score(&self, assignment_id: Uuid) -> AppResult<f64> {
        // Simple scoring: percentage of "yes" or positive responses
        // In a real implementation, this would be more sophisticated based on question weights
        let (total_weight, earned_weight): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(q.weight), 0) as total_weight,
                COALESCE(SUM(
                    CASE
                        WHEN q.question_type = 'yes_no' AND r.response_text = 'yes' THEN q.weight
                        WHEN q.question_type != 'yes_no' AND r.response_text IS NOT NULL THEN q.weight
                        ELSE 0
                    END
                ), 0) as earned_weight
            FROM questionnaire_questions q
            JOIN questionnaire_responses r ON q.id = r.question_id
            WHERE r.assignment_id = $1
            "#,
        )
        .bind(assignment_id)
        .fetch_one(&self.db)
        .await?;

        if total_weight == 0 {
            return Ok(100.0);
        }

        Ok((earned_weight as f64 / total_weight as f64) * 100.0)
    }

    // ==================== Statistics ====================

    /// Get questionnaire statistics
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<QuestionnaireStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_STATS, "summary");

        if let Some(cached) = self.cache.get::<QuestionnaireStats>(&cache_key).await? {
            return Ok(cached);
        }

        let (total_templates, published_templates): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'published') as published
            FROM questionnaire_templates
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let (total_assignments, pending_assignments, submitted_assignments, overdue_assignments): (
            i64,
            i64,
            i64,
            i64,
        ) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'pending' OR status = 'in_progress') as pending,
                COUNT(*) FILTER (WHERE status = 'submitted') as submitted,
                COUNT(*) FILTER (WHERE status IN ('pending', 'in_progress') AND due_date < CURRENT_DATE) as overdue
            FROM questionnaire_assignments
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let (avg_completion,): (Option<f64>,) = sqlx::query_as(
            r#"
            SELECT AVG(
                (SELECT COUNT(*) FROM questionnaire_responses r
                 WHERE r.assignment_id = qa.id AND r.response_text IS NOT NULL)::float /
                NULLIF((SELECT COUNT(*) FROM questionnaire_questions q WHERE q.template_id = qa.template_id), 0)::float
            ) * 100
            FROM questionnaire_assignments qa
            WHERE qa.organization_id = $1 AND qa.status IN ('in_progress', 'submitted')
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let stats = QuestionnaireStats {
            total_templates,
            published_templates,
            total_assignments,
            pending_assignments,
            submitted_assignments,
            overdue_assignments,
            average_completion_rate: avg_completion.unwrap_or(0.0),
        };

        self.cache
            .set(&cache_key, &stats, Some(Duration::from_secs(300)))
            .await?;

        Ok(stats)
    }

    // ==================== Cache Invalidation ====================

    async fn invalidate_template_cache(&self, org_id: Uuid, template_id: Uuid) -> AppResult<()> {
        let cache_key = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_TEMPLATE,
            &template_id.to_string(),
        );
        self.cache.delete(&cache_key).await?;
        self.invalidate_stats_cache(org_id).await
    }

    async fn invalidate_stats_cache(&self, org_id: Uuid) -> AppResult<()> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_STATS, "summary");
        self.cache.delete(&cache_key).await
    }
}

/// Generate a secure access token
fn generate_access_token() -> String {
    let mut rng = rand::thread_rng();
    let token: String = (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars[idx] as char
        })
        .collect();
    token
}

/// Calculate risk rating from score
fn calculate_risk_rating(score: f64) -> String {
    if score >= 90.0 {
        "low".to_string()
    } else if score >= 70.0 {
        "medium".to_string()
    } else if score >= 50.0 {
        "high".to_string()
    } else {
        "critical".to_string()
    }
}

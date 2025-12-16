use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    Audit, AuditFinding, AuditRequest, AuditRequestResponse, AuditStats, AuditTypeCount,
    AuditWithStats, CreateAudit, CreateAuditFinding, CreateAuditRequest, CreateRequestResponse,
    ListAuditsQuery, UpdateAudit, UpdateAuditFinding, AuditEvidenceItem, AuditEvidencePackage,
};
use crate::utils::{AppError, AppResult};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes
const CACHE_PREFIX_AUDIT: &str = "audit";
const CACHE_PREFIX_AUDIT_STATS: &str = "audit:stats";

#[derive(Clone)]
pub struct AuditService {
    db: PgPool,
    cache: CacheClient,
}

impl AuditService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Audit CRUD ====================

    /// List audits for an organization
    pub async fn list_audits(
        &self,
        org_id: Uuid,
        query: ListAuditsQuery,
    ) -> AppResult<Vec<AuditWithStats>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        let audits = if let Some(ref search) = query.search {
            let search_pattern = format!("%{}%", search.to_lowercase());
            sqlx::query_as::<_, Audit>(
                r#"
                SELECT id, organization_id, name, framework_id, audit_type, auditor_firm,
                       auditor_contact, period_start, period_end, status, created_at, updated_at
                FROM audits
                WHERE organization_id = $1
                  AND (LOWER(name) LIKE $2 OR LOWER(auditor_firm) LIKE $2)
                  AND ($3::text IS NULL OR status = $3)
                  AND ($4::text IS NULL OR audit_type = $4)
                  AND ($5::uuid IS NULL OR framework_id = $5)
                ORDER BY created_at DESC
                LIMIT $6 OFFSET $7
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .bind(&query.status)
            .bind(&query.audit_type)
            .bind(query.framework_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Audit>(
                r#"
                SELECT id, organization_id, name, framework_id, audit_type, auditor_firm,
                       auditor_contact, period_start, period_end, status, created_at, updated_at
                FROM audits
                WHERE organization_id = $1
                  AND ($2::text IS NULL OR status = $2)
                  AND ($3::text IS NULL OR audit_type = $3)
                  AND ($4::uuid IS NULL OR framework_id = $4)
                ORDER BY created_at DESC
                LIMIT $5 OFFSET $6
                "#,
            )
            .bind(org_id)
            .bind(&query.status)
            .bind(&query.audit_type)
            .bind(query.framework_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        // Get request and finding counts
        let audit_ids: Vec<Uuid> = audits.iter().map(|a| a.id).collect();

        let request_counts: Vec<(Uuid, i64, i64)> = if !audit_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT audit_id, COUNT(*) as total,
                       COUNT(*) FILTER (WHERE status = 'open') as open_count
                FROM audit_requests
                WHERE audit_id = ANY($1)
                GROUP BY audit_id
                "#,
            )
            .bind(&audit_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let finding_counts: Vec<(Uuid, i64, i64)> = if !audit_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT audit_id, COUNT(*) as total,
                       COUNT(*) FILTER (WHERE status = 'open') as open_count
                FROM audit_findings
                WHERE audit_id = ANY($1)
                GROUP BY audit_id
                "#,
            )
            .bind(&audit_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let req_map: std::collections::HashMap<Uuid, (i64, i64)> =
            request_counts.into_iter().map(|(id, t, o)| (id, (t, o))).collect();
        let find_map: std::collections::HashMap<Uuid, (i64, i64)> =
            finding_counts.into_iter().map(|(id, t, o)| (id, (t, o))).collect();

        let result: Vec<AuditWithStats> = audits
            .into_iter()
            .map(|audit| {
                let (req_count, open_req) = req_map.get(&audit.id).copied().unwrap_or((0, 0));
                let (find_count, open_find) = find_map.get(&audit.id).copied().unwrap_or((0, 0));
                AuditWithStats {
                    audit,
                    request_count: req_count,
                    open_requests: open_req,
                    finding_count: find_count,
                    open_findings: open_find,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get a single audit by ID
    pub async fn get_audit(&self, org_id: Uuid, id: Uuid) -> AppResult<AuditWithStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_AUDIT, &id.to_string());

        if let Some(cached) = self.cache.get::<AuditWithStats>(&cache_key).await? {
            tracing::debug!("Cache hit for audit {}", id);
            return Ok(cached);
        }

        let audit = sqlx::query_as::<_, Audit>(
            r#"
            SELECT id, organization_id, name, framework_id, audit_type, auditor_firm,
                   auditor_contact, period_start, period_end, status, created_at, updated_at
            FROM audits
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Audit {} not found", id)))?;

        let (request_count, open_requests): (i64, i64) = sqlx::query_as(
            r#"
            SELECT COUNT(*), COUNT(*) FILTER (WHERE status = 'open')
            FROM audit_requests WHERE audit_id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        let (finding_count, open_findings): (i64, i64) = sqlx::query_as(
            r#"
            SELECT COUNT(*), COUNT(*) FILTER (WHERE status = 'open')
            FROM audit_findings WHERE audit_id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        let result = AuditWithStats {
            audit,
            request_count,
            open_requests,
            finding_count,
            open_findings,
        };

        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create a new audit
    pub async fn create_audit(&self, org_id: Uuid, input: CreateAudit) -> AppResult<Audit> {
        Audit::validate_create(&input).map_err(AppError::ValidationError)?;

        let audit = sqlx::query_as::<_, Audit>(
            r#"
            INSERT INTO audits (organization_id, name, framework_id, audit_type, auditor_firm,
                                auditor_contact, period_start, period_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, organization_id, name, framework_id, audit_type, auditor_firm,
                      auditor_contact, period_start, period_end, status, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(&input.name)
        .bind(input.framework_id)
        .bind(&input.audit_type)
        .bind(&input.auditor_firm)
        .bind(&input.auditor_contact)
        .bind(input.period_start)
        .bind(input.period_end)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_org_audit_caches(org_id).await?;

        tracing::info!("Created audit: {} ({})", audit.name, audit.id);

        Ok(audit)
    }

    /// Update an audit
    pub async fn update_audit(
        &self,
        org_id: Uuid,
        id: Uuid,
        input: UpdateAudit,
    ) -> AppResult<Audit> {
        self.get_audit(org_id, id).await?;

        let audit = sqlx::query_as::<_, Audit>(
            r#"
            UPDATE audits
            SET
                name = COALESCE($3, name),
                framework_id = COALESCE($4, framework_id),
                audit_type = COALESCE($5, audit_type),
                auditor_firm = COALESCE($6, auditor_firm),
                auditor_contact = COALESCE($7, auditor_contact),
                period_start = COALESCE($8, period_start),
                period_end = COALESCE($9, period_end),
                status = COALESCE($10, status),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, name, framework_id, audit_type, auditor_firm,
                      auditor_contact, period_start, period_end, status, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(&input.name)
        .bind(input.framework_id)
        .bind(&input.audit_type)
        .bind(&input.auditor_firm)
        .bind(&input.auditor_contact)
        .bind(input.period_start)
        .bind(input.period_end)
        .bind(&input.status)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_audit_cache(org_id, id).await?;

        tracing::info!("Updated audit: {} ({})", audit.name, audit.id);

        Ok(audit)
    }

    /// Delete an audit
    pub async fn delete_audit(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        self.get_audit(org_id, id).await?;

        sqlx::query("DELETE FROM audits WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        self.invalidate_audit_cache(org_id, id).await?;

        tracing::info!("Deleted audit: {}", id);

        Ok(())
    }

    // ==================== Audit Requests ====================

    /// List requests for an audit
    pub async fn list_requests(&self, org_id: Uuid, audit_id: Uuid) -> AppResult<Vec<AuditRequest>> {
        self.get_audit(org_id, audit_id).await?;

        let requests = sqlx::query_as::<_, AuditRequest>(
            r#"
            SELECT id, audit_id, request_type, title, description, status,
                   assigned_to, due_at, created_at, updated_at
            FROM audit_requests
            WHERE audit_id = $1
            ORDER BY due_at ASC NULLS LAST, created_at DESC
            "#,
        )
        .bind(audit_id)
        .fetch_all(&self.db)
        .await?;

        Ok(requests)
    }

    /// Create an audit request
    pub async fn create_request(
        &self,
        org_id: Uuid,
        audit_id: Uuid,
        input: CreateAuditRequest,
    ) -> AppResult<AuditRequest> {
        self.get_audit(org_id, audit_id).await?;

        let request = sqlx::query_as::<_, AuditRequest>(
            r#"
            INSERT INTO audit_requests (audit_id, request_type, title, description, assigned_to, due_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, audit_id, request_type, title, description, status,
                      assigned_to, due_at, created_at, updated_at
            "#,
        )
        .bind(audit_id)
        .bind(&input.request_type)
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.assigned_to)
        .bind(input.due_at)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_audit_cache(org_id, audit_id).await?;

        Ok(request)
    }

    /// Add response to an audit request
    pub async fn add_response(
        &self,
        org_id: Uuid,
        audit_id: Uuid,
        request_id: Uuid,
        user_id: Option<Uuid>,
        input: CreateRequestResponse,
    ) -> AppResult<AuditRequestResponse> {
        self.get_audit(org_id, audit_id).await?;

        let response = sqlx::query_as::<_, AuditRequestResponse>(
            r#"
            INSERT INTO audit_request_responses (audit_request_id, response_text, evidence_ids, responded_by)
            VALUES ($1, $2, $3, $4)
            RETURNING id, audit_request_id, response_text, evidence_ids, responded_by, responded_at, created_at
            "#,
        )
        .bind(request_id)
        .bind(&input.response_text)
        .bind(&input.evidence_ids)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Mark request as responded
        sqlx::query("UPDATE audit_requests SET status = 'responded', updated_at = NOW() WHERE id = $1")
            .bind(request_id)
            .execute(&self.db)
            .await?;

        self.invalidate_audit_cache(org_id, audit_id).await?;

        Ok(response)
    }

    // ==================== Audit Findings ====================

    /// List findings for an audit
    pub async fn list_findings(&self, org_id: Uuid, audit_id: Uuid) -> AppResult<Vec<AuditFinding>> {
        self.get_audit(org_id, audit_id).await?;

        let findings = sqlx::query_as::<_, AuditFinding>(
            r#"
            SELECT id, audit_id, finding_type, title, description, recommendation,
                   control_ids, status, remediation_plan, remediation_due, created_at, updated_at
            FROM audit_findings
            WHERE audit_id = $1
            ORDER BY remediation_due ASC NULLS LAST, created_at DESC
            "#,
        )
        .bind(audit_id)
        .fetch_all(&self.db)
        .await?;

        Ok(findings)
    }

    /// Create an audit finding
    pub async fn create_finding(
        &self,
        org_id: Uuid,
        audit_id: Uuid,
        input: CreateAuditFinding,
    ) -> AppResult<AuditFinding> {
        self.get_audit(org_id, audit_id).await?;

        let finding = sqlx::query_as::<_, AuditFinding>(
            r#"
            INSERT INTO audit_findings (audit_id, finding_type, title, description, recommendation,
                                        control_ids, remediation_plan, remediation_due)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, audit_id, finding_type, title, description, recommendation,
                      control_ids, status, remediation_plan, remediation_due, created_at, updated_at
            "#,
        )
        .bind(audit_id)
        .bind(&input.finding_type)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.recommendation)
        .bind(&input.control_ids)
        .bind(&input.remediation_plan)
        .bind(input.remediation_due)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_audit_cache(org_id, audit_id).await?;

        Ok(finding)
    }

    /// Get a specific audit finding
    pub async fn get_finding(
        &self,
        org_id: Uuid,
        audit_id: Uuid,
        finding_id: Uuid,
    ) -> AppResult<AuditFinding> {
        // Verify audit exists and belongs to org
        self.get_audit(org_id, audit_id).await?;

        let finding = sqlx::query_as::<_, AuditFinding>(
            r#"
            SELECT id, audit_id, finding_type, title, description, recommendation,
                   control_ids, status, remediation_plan, remediation_due, created_at, updated_at
            FROM audit_findings
            WHERE id = $1 AND audit_id = $2
            "#,
        )
        .bind(finding_id)
        .bind(audit_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Finding not found".to_string()))?;

        Ok(finding)
    }

    /// Update an audit finding
    pub async fn update_finding(
        &self,
        org_id: Uuid,
        audit_id: Uuid,
        finding_id: Uuid,
        input: UpdateAuditFinding,
    ) -> AppResult<AuditFinding> {
        self.get_audit(org_id, audit_id).await?;

        let finding = sqlx::query_as::<_, AuditFinding>(
            r#"
            UPDATE audit_findings
            SET
                finding_type = COALESCE($3, finding_type),
                title = COALESCE($4, title),
                description = COALESCE($5, description),
                recommendation = COALESCE($6, recommendation),
                control_ids = COALESCE($7, control_ids),
                status = COALESCE($8, status),
                remediation_plan = COALESCE($9, remediation_plan),
                remediation_due = COALESCE($10, remediation_due),
                updated_at = NOW()
            WHERE id = $1 AND audit_id = $2
            RETURNING id, audit_id, finding_type, title, description, recommendation,
                      control_ids, status, remediation_plan, remediation_due, created_at, updated_at
            "#,
        )
        .bind(finding_id)
        .bind(audit_id)
        .bind(&input.finding_type)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.recommendation)
        .bind(&input.control_ids)
        .bind(&input.status)
        .bind(&input.remediation_plan)
        .bind(input.remediation_due)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_audit_cache(org_id, audit_id).await?;

        Ok(finding)
    }

    // ==================== Statistics ====================

    /// Get audit statistics
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<AuditStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_AUDIT_STATS, "summary");

        if let Some(cached) = self.cache.get::<AuditStats>(&cache_key).await? {
            tracing::debug!("Cache hit for audit stats");
            return Ok(cached);
        }

        let (total, in_progress, completed): (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status IN ('in_progress', 'fieldwork', 'reporting')) as in_progress,
                COUNT(*) FILTER (WHERE status = 'completed') as completed
            FROM audits
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let by_type: Vec<AuditTypeCount> = sqlx::query_as(
            r#"
            SELECT audit_type, COUNT(*) as count
            FROM audits
            WHERE organization_id = $1
            GROUP BY audit_type
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let (open_findings,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM audit_findings af
            JOIN audits a ON af.audit_id = a.id
            WHERE a.organization_id = $1 AND af.status = 'open'
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let (overdue_requests,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM audit_requests ar
            JOIN audits a ON ar.audit_id = a.id
            WHERE a.organization_id = $1 AND ar.status = 'open' AND ar.due_at < NOW()
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let stats = AuditStats {
            total,
            in_progress,
            completed,
            by_type,
            open_findings,
            overdue_requests,
        };

        self.cache
            .set(&cache_key, &stats, Some(Duration::from_secs(300)))
            .await?;

        Ok(stats)
    }

    // ==================== Evidence Packaging ====================

    /// Get evidence package for an audit
    /// Returns evidence linked to controls that map to the audit's framework requirements,
    /// as well as evidence directly linked to audit request responses.
    pub async fn get_evidence_package(
        &self,
        org_id: Uuid,
        audit_id: Uuid,
    ) -> AppResult<AuditEvidencePackage> {
        // Get audit details
        let audit = self.get_audit(org_id, audit_id).await?;

        // Get framework name if audit has a framework
        let framework_name: Option<String> = if let Some(framework_id) = audit.audit.framework_id {
            sqlx::query_scalar("SELECT name FROM frameworks WHERE id = $1")
                .bind(framework_id)
                .fetch_optional(&self.db)
                .await?
        } else {
            None
        };

        // Get evidence linked via controls mapped to the audit's framework requirements
        let framework_evidence: Vec<(Uuid, String, Option<String>, String, String, Option<String>, Option<i64>, Option<String>, chrono::DateTime<chrono::Utc>, String)> =
            if audit.audit.framework_id.is_some() {
                sqlx::query_as(
                    r#"
                    SELECT DISTINCT e.id, e.title, e.description, e.evidence_type, e.source,
                           e.file_path, e.file_size, e.mime_type, e.collected_at,
                           c.code as control_code
                    FROM evidence e
                    JOIN evidence_control_links ecl ON e.id = ecl.evidence_id
                    JOIN controls c ON ecl.control_id = c.id
                    JOIN control_requirement_mappings crm ON c.id = crm.control_id
                    JOIN framework_requirements fr ON crm.requirement_id = fr.id
                    WHERE fr.framework_id = $1 AND e.organization_id = $2
                    ORDER BY e.collected_at DESC
                    "#,
                )
                .bind(audit.audit.framework_id)
                .bind(org_id)
                .fetch_all(&self.db)
                .await?
            } else {
                vec![]
            };

        // Get evidence directly linked to audit request responses
        let request_evidence: Vec<(Uuid, String, Option<String>, String, String, Option<String>, Option<i64>, Option<String>, chrono::DateTime<chrono::Utc>, String)> =
            sqlx::query_as(
                r#"
                SELECT DISTINCT e.id, e.title, e.description, e.evidence_type, e.source,
                       e.file_path, e.file_size, e.mime_type, e.collected_at,
                       ar.title as request_title
                FROM evidence e
                JOIN audit_request_responses arr ON e.id = ANY(arr.evidence_ids)
                JOIN audit_requests ar ON arr.audit_request_id = ar.id
                WHERE ar.audit_id = $1 AND e.organization_id = $2
                ORDER BY e.collected_at DESC
                "#,
            )
            .bind(audit_id)
            .bind(org_id)
            .fetch_all(&self.db)
            .await?;

        // Combine and deduplicate evidence
        let mut evidence_map: std::collections::HashMap<Uuid, AuditEvidenceItem> =
            std::collections::HashMap::new();

        for (id, title, desc, etype, source, path, size, mime, collected, control_code) in
            framework_evidence
        {
            let entry = evidence_map.entry(id).or_insert_with(|| AuditEvidenceItem {
                id,
                title,
                description: desc,
                evidence_type: etype,
                source,
                file_path: path,
                file_size: size,
                mime_type: mime,
                collected_at: collected,
                linked_controls: vec![],
                linked_requests: vec![],
            });
            if !entry.linked_controls.contains(&control_code) {
                entry.linked_controls.push(control_code);
            }
        }

        for (id, title, desc, etype, source, path, size, mime, collected, request_title) in
            request_evidence
        {
            let entry = evidence_map.entry(id).or_insert_with(|| AuditEvidenceItem {
                id,
                title,
                description: desc,
                evidence_type: etype,
                source,
                file_path: path,
                file_size: size,
                mime_type: mime,
                collected_at: collected,
                linked_controls: vec![],
                linked_requests: vec![],
            });
            if !entry.linked_requests.contains(&request_title) {
                entry.linked_requests.push(request_title);
            }
        }

        let evidence: Vec<AuditEvidenceItem> = evidence_map.into_values().collect();
        let total_file_size: i64 = evidence.iter().filter_map(|e| e.file_size).sum();

        Ok(AuditEvidencePackage {
            audit_id,
            audit_name: audit.audit.name,
            framework_name,
            period_start: audit.audit.period_start,
            period_end: audit.audit.period_end,
            evidence_count: evidence.len(),
            total_file_size,
            evidence,
            generated_at: chrono::Utc::now(),
        })
    }

    // ==================== Cache Invalidation ====================

    async fn invalidate_audit_cache(&self, org_id: Uuid, audit_id: Uuid) -> AppResult<()> {
        let cache_key = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_AUDIT,
            &audit_id.to_string(),
        );
        self.cache.delete(&cache_key).await?;

        self.invalidate_org_audit_caches(org_id).await
    }

    async fn invalidate_org_audit_caches(&self, org_id: Uuid) -> AppResult<()> {
        let stats_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_AUDIT_STATS, "summary");
        self.cache.delete(&stats_key).await
    }
}

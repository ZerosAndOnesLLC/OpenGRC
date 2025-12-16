use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    AuditFinding, CreateTask, CreateTaskComment, ListTasksQuery, Task, TaskAssigneeCount,
    TaskComment, TaskCommentWithUser, TaskPriorityCount, TaskRecurrenceHistory, TaskStats,
    TaskTypeCount, TaskWithAssignee, UpdateTask,
};
use crate::utils::{AppError, AppResult};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc, Weekday};
use sqlx::PgPool;
use std::time::Duration as StdDuration;
use uuid::Uuid;

const CACHE_TTL: StdDuration = StdDuration::from_secs(900); // 15 minutes
const CACHE_PREFIX_TASK: &str = "task";
const CACHE_PREFIX_TASK_STATS: &str = "task:stats";

#[derive(Clone)]
pub struct TaskService {
    db: PgPool,
    cache: CacheClient,
}

impl TaskService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    /// Invalidate task caches for an organization
    async fn invalidate_caches(&self, org_id: Uuid) {
        let pattern = org_cache_key(&org_id.to_string(), CACHE_PREFIX_TASK, "*");
        let _ = self.cache.delete_pattern(&pattern).await;
        let stats_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_TASK_STATS, "");
        let _ = self.cache.delete(&stats_key).await;
    }

    // ==================== Task CRUD ====================

    /// List tasks for an organization
    pub async fn list_tasks(
        &self,
        org_id: Uuid,
        query: ListTasksQuery,
    ) -> AppResult<Vec<TaskWithAssignee>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        let tasks = if let Some(ref search) = query.search {
            let search_pattern = format!("%{}%", search.to_lowercase());
            sqlx::query_as::<_, Task>(
                r#"
                SELECT t.id, t.organization_id, t.title, t.description, t.task_type,
                       t.related_entity_type, t.related_entity_id, t.assignee_id,
                       t.due_at, t.completed_at, t.status, t.priority,
                       t.created_by, t.created_at, t.updated_at,
                       t.is_recurring, t.recurrence_pattern, t.recurrence_interval,
                       t.recurrence_day_of_week, t.recurrence_day_of_month, t.recurrence_month_of_year,
                       t.recurrence_end_at, t.recurrence_count, t.recurrence_occurrences,
                       t.parent_task_id, t.next_occurrence_at, t.last_occurrence_at
                FROM tasks t
                WHERE t.organization_id = $1
                  AND (LOWER(t.title) LIKE $2 OR LOWER(t.description) LIKE $2)
                  AND ($3::text IS NULL OR t.status = $3)
                  AND ($4::text IS NULL OR t.task_type = $4)
                  AND ($5::text IS NULL OR t.priority = $5)
                  AND ($6::uuid IS NULL OR t.assignee_id = $6)
                  AND ($7::text IS NULL OR t.related_entity_type = $7)
                  AND ($8::uuid IS NULL OR t.related_entity_id = $8)
                  AND ($9::bool IS NULL OR $9 = false OR (t.due_at IS NOT NULL AND t.due_at < NOW() AND t.status != 'completed'))
                  AND ($10::bool IS NULL OR $10 = false OR t.is_recurring = true)
                  AND ($11::bool IS NULL OR $11 = false OR t.is_recurring = false)
                  AND ($12::uuid IS NULL OR t.parent_task_id = $12)
                ORDER BY
                    CASE WHEN t.status = 'open' AND t.due_at IS NOT NULL AND t.due_at < NOW() THEN 0
                         WHEN t.priority = 'critical' THEN 1
                         WHEN t.priority = 'high' THEN 2
                         WHEN t.priority = 'medium' THEN 3
                         ELSE 4 END,
                    t.due_at ASC NULLS LAST,
                    t.created_at DESC
                LIMIT $13 OFFSET $14
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .bind(&query.status)
            .bind(&query.task_type)
            .bind(&query.priority)
            .bind(query.assignee_id)
            .bind(&query.related_entity_type)
            .bind(query.related_entity_id)
            .bind(query.overdue_only)
            .bind(query.recurring_only)
            .bind(query.exclude_recurring_templates)
            .bind(query.parent_task_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Task>(
                r#"
                SELECT t.id, t.organization_id, t.title, t.description, t.task_type,
                       t.related_entity_type, t.related_entity_id, t.assignee_id,
                       t.due_at, t.completed_at, t.status, t.priority,
                       t.created_by, t.created_at, t.updated_at,
                       t.is_recurring, t.recurrence_pattern, t.recurrence_interval,
                       t.recurrence_day_of_week, t.recurrence_day_of_month, t.recurrence_month_of_year,
                       t.recurrence_end_at, t.recurrence_count, t.recurrence_occurrences,
                       t.parent_task_id, t.next_occurrence_at, t.last_occurrence_at
                FROM tasks t
                WHERE t.organization_id = $1
                  AND ($2::text IS NULL OR t.status = $2)
                  AND ($3::text IS NULL OR t.task_type = $3)
                  AND ($4::text IS NULL OR t.priority = $4)
                  AND ($5::uuid IS NULL OR t.assignee_id = $5)
                  AND ($6::text IS NULL OR t.related_entity_type = $6)
                  AND ($7::uuid IS NULL OR t.related_entity_id = $7)
                  AND ($8::bool IS NULL OR $8 = false OR (t.due_at IS NOT NULL AND t.due_at < NOW() AND t.status != 'completed'))
                  AND ($9::bool IS NULL OR $9 = false OR t.is_recurring = true)
                  AND ($10::bool IS NULL OR $10 = false OR t.is_recurring = false)
                  AND ($11::uuid IS NULL OR t.parent_task_id = $11)
                ORDER BY
                    CASE WHEN t.status = 'open' AND t.due_at IS NOT NULL AND t.due_at < NOW() THEN 0
                         WHEN t.priority = 'critical' THEN 1
                         WHEN t.priority = 'high' THEN 2
                         WHEN t.priority = 'medium' THEN 3
                         ELSE 4 END,
                    t.due_at ASC NULLS LAST,
                    t.created_at DESC
                LIMIT $12 OFFSET $13
                "#,
            )
            .bind(org_id)
            .bind(&query.status)
            .bind(&query.task_type)
            .bind(&query.priority)
            .bind(query.assignee_id)
            .bind(&query.related_entity_type)
            .bind(query.related_entity_id)
            .bind(query.overdue_only)
            .bind(query.recurring_only)
            .bind(query.exclude_recurring_templates)
            .bind(query.parent_task_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        // Get assignee and creator info
        let user_ids: Vec<Uuid> = tasks
            .iter()
            .flat_map(|t| vec![t.assignee_id, t.created_by].into_iter().flatten())
            .collect();

        let users: Vec<(Uuid, String, String)> = if !user_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT id, name, email
                FROM users
                WHERE id = ANY($1)
                "#,
            )
            .bind(&user_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let user_map: std::collections::HashMap<Uuid, (String, String)> = users
            .into_iter()
            .map(|(id, name, email)| (id, (name, email)))
            .collect();

        let result: Vec<TaskWithAssignee> = tasks
            .into_iter()
            .map(|task| {
                let (assignee_name, assignee_email) = task
                    .assignee_id
                    .and_then(|id| user_map.get(&id).cloned())
                    .map(|(n, e)| (Some(n), Some(e)))
                    .unwrap_or((None, None));
                let created_by_name = task
                    .created_by
                    .and_then(|id| user_map.get(&id).map(|(n, _)| n.clone()));
                TaskWithAssignee {
                    task,
                    assignee_name,
                    assignee_email,
                    created_by_name,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get a single task by ID
    pub async fn get_task(&self, org_id: Uuid, id: Uuid) -> AppResult<TaskWithAssignee> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_TASK, &id.to_string());

        if let Some(cached) = self.cache.get::<TaskWithAssignee>(&cache_key).await? {
            return Ok(cached);
        }

        let task = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, organization_id, title, description, task_type,
                   related_entity_type, related_entity_id, assignee_id,
                   due_at, completed_at, status, priority,
                   created_by, created_at, updated_at,
                   is_recurring, recurrence_pattern, recurrence_interval,
                   recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                   recurrence_end_at, recurrence_count, recurrence_occurrences,
                   parent_task_id, next_occurrence_at, last_occurrence_at
            FROM tasks
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        let (assignee_name, assignee_email) = if let Some(assignee_id) = task.assignee_id {
            sqlx::query_as::<_, (String, String)>(
                "SELECT name, email FROM users WHERE id = $1",
            )
            .bind(assignee_id)
            .fetch_optional(&self.db)
            .await?
            .map(|(n, e)| (Some(n), Some(e)))
            .unwrap_or((None, None))
        } else {
            (None, None)
        };

        let created_by_name = if let Some(created_by) = task.created_by {
            sqlx::query_scalar::<_, String>("SELECT name FROM users WHERE id = $1")
                .bind(created_by)
                .fetch_optional(&self.db)
                .await?
        } else {
            None
        };

        let result = TaskWithAssignee {
            task,
            assignee_name,
            assignee_email,
            created_by_name,
        };

        let _ = self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await;
        Ok(result)
    }

    /// Create a new task
    pub async fn create_task(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        input: CreateTask,
    ) -> AppResult<Task> {
        Task::validate_create(&input).map_err(AppError::BadRequest)?;

        let is_recurring = input.is_recurring.unwrap_or(false);
        let next_occurrence_at = if is_recurring {
            input.due_at.or_else(|| Some(Utc::now()))
        } else {
            None
        };

        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (organization_id, title, description, task_type,
                              related_entity_type, related_entity_id, assignee_id,
                              due_at, priority, created_by, status,
                              is_recurring, recurrence_pattern, recurrence_interval,
                              recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                              recurrence_end_at, recurrence_count, recurrence_occurrences,
                              next_occurrence_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'open',
                    $11, $12, $13, $14, $15, $16, $17, $18, 0, $19)
            RETURNING id, organization_id, title, description, task_type,
                      related_entity_type, related_entity_id, assignee_id,
                      due_at, completed_at, status, priority,
                      created_by, created_at, updated_at,
                      is_recurring, recurrence_pattern, recurrence_interval,
                      recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                      recurrence_end_at, recurrence_count, recurrence_occurrences,
                      parent_task_id, next_occurrence_at, last_occurrence_at
            "#,
        )
        .bind(org_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.task_type.as_deref().unwrap_or("general"))
        .bind(&input.related_entity_type)
        .bind(input.related_entity_id)
        .bind(input.assignee_id)
        .bind(input.due_at)
        .bind(input.priority.as_deref().unwrap_or("medium"))
        .bind(user_id)
        .bind(is_recurring)
        .bind(&input.recurrence_pattern)
        .bind(input.recurrence_interval)
        .bind(input.recurrence_day_of_week)
        .bind(input.recurrence_day_of_month)
        .bind(input.recurrence_month_of_year)
        .bind(input.recurrence_end_at)
        .bind(input.recurrence_count)
        .bind(next_occurrence_at)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_caches(org_id).await;
        Ok(task)
    }

    /// Create a remediation task from an audit finding
    pub async fn create_remediation_task(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        finding: &AuditFinding,
        assignee_id: Option<Uuid>,
        priority: String,
    ) -> AppResult<Task> {
        // Build title and description from finding
        let title = format!("Remediate: {}", finding.title);
        let description = format!(
            "Remediation task for audit finding:\n\n**Finding:** {}\n\n**Description:** {}\n\n**Recommendation:** {}",
            finding.title,
            finding.description.as_deref().unwrap_or("No description provided"),
            finding.recommendation.as_deref().unwrap_or("No recommendation provided")
        );

        // Use finding's remediation_due date if available
        let due_at = finding.remediation_due.map(|d| {
            d.and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(chrono::Utc)
                .unwrap()
        });

        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (organization_id, title, description, task_type,
                              related_entity_type, related_entity_id, assignee_id,
                              due_at, priority, created_by, status)
            VALUES ($1, $2, $3, 'remediation', 'audit_finding', $4, $5, $6, $7, $8, 'open')
            RETURNING id, organization_id, title, description, task_type,
                      related_entity_type, related_entity_id, assignee_id,
                      due_at, completed_at, status, priority,
                      created_by, created_at, updated_at,
                      is_recurring, recurrence_pattern, recurrence_interval,
                      recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                      recurrence_end_at, recurrence_count, recurrence_occurrences,
                      parent_task_id, next_occurrence_at, last_occurrence_at
            "#,
        )
        .bind(org_id)
        .bind(&title)
        .bind(&description)
        .bind(finding.id)
        .bind(assignee_id)
        .bind(due_at)
        .bind(&priority)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Update finding status to 'in_remediation'
        sqlx::query("UPDATE audit_findings SET status = 'in_remediation', updated_at = NOW() WHERE id = $1")
            .bind(finding.id)
            .execute(&self.db)
            .await?;

        self.invalidate_caches(org_id).await;
        Ok(task)
    }

    /// Update a task
    pub async fn update_task(
        &self,
        org_id: Uuid,
        id: Uuid,
        input: UpdateTask,
    ) -> AppResult<Task> {
        // Check task exists
        let _existing = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM tasks WHERE id = $1 AND organization_id = $2",
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        // Set completed_at if status changed to completed
        let completed_at = if input.status.as_deref() == Some("completed") {
            Some(Utc::now())
        } else {
            None
        };

        let task = sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks SET
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                task_type = COALESCE($5, task_type),
                related_entity_type = COALESCE($6, related_entity_type),
                related_entity_id = COALESCE($7, related_entity_id),
                assignee_id = COALESCE($8, assignee_id),
                due_at = COALESCE($9, due_at),
                status = COALESCE($10, status),
                priority = COALESCE($11, priority),
                completed_at = COALESCE($12, completed_at),
                is_recurring = COALESCE($13, is_recurring),
                recurrence_pattern = COALESCE($14, recurrence_pattern),
                recurrence_interval = COALESCE($15, recurrence_interval),
                recurrence_day_of_week = COALESCE($16, recurrence_day_of_week),
                recurrence_day_of_month = COALESCE($17, recurrence_day_of_month),
                recurrence_month_of_year = COALESCE($18, recurrence_month_of_year),
                recurrence_end_at = COALESCE($19, recurrence_end_at),
                recurrence_count = COALESCE($20, recurrence_count),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, title, description, task_type,
                      related_entity_type, related_entity_id, assignee_id,
                      due_at, completed_at, status, priority,
                      created_by, created_at, updated_at,
                      is_recurring, recurrence_pattern, recurrence_interval,
                      recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                      recurrence_end_at, recurrence_count, recurrence_occurrences,
                      parent_task_id, next_occurrence_at, last_occurrence_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.task_type)
        .bind(&input.related_entity_type)
        .bind(input.related_entity_id)
        .bind(input.assignee_id)
        .bind(input.due_at)
        .bind(&input.status)
        .bind(&input.priority)
        .bind(completed_at)
        .bind(input.is_recurring)
        .bind(&input.recurrence_pattern)
        .bind(input.recurrence_interval)
        .bind(input.recurrence_day_of_week)
        .bind(input.recurrence_day_of_month)
        .bind(input.recurrence_month_of_year)
        .bind(input.recurrence_end_at)
        .bind(input.recurrence_count)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_caches(org_id).await;
        Ok(task)
    }

    /// Delete a task
    pub async fn delete_task(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Task not found".to_string()));
        }

        self.invalidate_caches(org_id).await;
        Ok(())
    }

    /// Complete a task
    pub async fn complete_task(&self, org_id: Uuid, id: Uuid) -> AppResult<Task> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks SET
                status = 'completed',
                completed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, title, description, task_type,
                      related_entity_type, related_entity_id, assignee_id,
                      due_at, completed_at, status, priority,
                      created_by, created_at, updated_at,
                      is_recurring, recurrence_pattern, recurrence_interval,
                      recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                      recurrence_end_at, recurrence_count, recurrence_occurrences,
                      parent_task_id, next_occurrence_at, last_occurrence_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        self.invalidate_caches(org_id).await;
        Ok(task)
    }

    // ==================== Task Statistics ====================

    /// Get task statistics for an organization
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<TaskStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_TASK_STATS, "");

        if let Some(cached) = self.cache.get::<TaskStats>(&cache_key).await? {
            return Ok(cached);
        }

        let (total, open, in_progress, completed, overdue): (i64, i64, i64, i64, i64) =
            sqlx::query_as(
                r#"
                SELECT
                    COUNT(*),
                    COUNT(*) FILTER (WHERE status = 'open'),
                    COUNT(*) FILTER (WHERE status = 'in_progress'),
                    COUNT(*) FILTER (WHERE status = 'completed'),
                    COUNT(*) FILTER (WHERE status != 'completed' AND due_at IS NOT NULL AND due_at < NOW())
                FROM tasks
                WHERE organization_id = $1
                "#,
            )
            .bind(org_id)
            .fetch_one(&self.db)
            .await?;

        let by_type: Vec<TaskTypeCount> = sqlx::query_as(
            r#"
            SELECT task_type, COUNT(*) as count
            FROM tasks
            WHERE organization_id = $1
            GROUP BY task_type
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let by_priority: Vec<TaskPriorityCount> = sqlx::query_as(
            r#"
            SELECT priority, COUNT(*) as count
            FROM tasks
            WHERE organization_id = $1 AND status != 'completed'
            GROUP BY priority
            ORDER BY
                CASE priority
                    WHEN 'critical' THEN 1
                    WHEN 'high' THEN 2
                    WHEN 'medium' THEN 3
                    WHEN 'low' THEN 4
                    ELSE 5
                END
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let by_assignee: Vec<TaskAssigneeCount> = sqlx::query_as(
            r#"
            SELECT t.assignee_id, u.name as assignee_name, COUNT(*) as count
            FROM tasks t
            LEFT JOIN users u ON t.assignee_id = u.id
            WHERE t.organization_id = $1 AND t.status != 'completed'
            GROUP BY t.assignee_id, u.name
            ORDER BY count DESC
            LIMIT 10
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let now = Utc::now();
        let end_of_today = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
        let end_of_week = (now + Duration::days(7)).date_naive().and_hms_opt(23, 59, 59).unwrap();

        let (due_today, due_this_week): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE due_at::date = $2::date),
                COUNT(*) FILTER (WHERE due_at::date <= $3::date)
            FROM tasks
            WHERE organization_id = $1 AND status != 'completed' AND due_at IS NOT NULL
            "#,
        )
        .bind(org_id)
        .bind(end_of_today)
        .bind(end_of_week)
        .fetch_one(&self.db)
        .await?;

        let stats = TaskStats {
            total,
            open,
            in_progress,
            completed,
            overdue,
            by_type,
            by_priority,
            by_assignee,
            due_today,
            due_this_week,
        };

        let _ = self.cache.set(&cache_key, &stats, Some(CACHE_TTL)).await;
        Ok(stats)
    }

    // ==================== My Tasks ====================

    /// Get tasks assigned to a specific user
    pub async fn get_my_tasks(&self, org_id: Uuid, user_id: Uuid) -> AppResult<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, organization_id, title, description, task_type,
                   related_entity_type, related_entity_id, assignee_id,
                   due_at, completed_at, status, priority,
                   created_by, created_at, updated_at,
                   is_recurring, recurrence_pattern, recurrence_interval,
                   recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                   recurrence_end_at, recurrence_count, recurrence_occurrences,
                   parent_task_id, next_occurrence_at, last_occurrence_at
            FROM tasks
            WHERE organization_id = $1 AND assignee_id = $2 AND status != 'completed'
              AND is_recurring = false
            ORDER BY
                CASE WHEN due_at IS NOT NULL AND due_at < NOW() THEN 0
                     WHEN priority = 'critical' THEN 1
                     WHEN priority = 'high' THEN 2
                     WHEN priority = 'medium' THEN 3
                     ELSE 4 END,
                due_at ASC NULLS LAST
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(tasks)
    }

    /// Get overdue tasks for an organization
    pub async fn get_overdue_tasks(&self, org_id: Uuid) -> AppResult<Vec<TaskWithAssignee>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, organization_id, title, description, task_type,
                   related_entity_type, related_entity_id, assignee_id,
                   due_at, completed_at, status, priority,
                   created_by, created_at, updated_at,
                   is_recurring, recurrence_pattern, recurrence_interval,
                   recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                   recurrence_end_at, recurrence_count, recurrence_occurrences,
                   parent_task_id, next_occurrence_at, last_occurrence_at
            FROM tasks
            WHERE organization_id = $1
              AND status != 'completed'
              AND due_at IS NOT NULL
              AND due_at < NOW()
              AND is_recurring = false
            ORDER BY due_at ASC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        // Get assignee info
        let user_ids: Vec<Uuid> = tasks.iter().filter_map(|t| t.assignee_id).collect();

        let users: Vec<(Uuid, String, String)> = if !user_ids.is_empty() {
            sqlx::query_as("SELECT id, name, email FROM users WHERE id = ANY($1)")
                .bind(&user_ids)
                .fetch_all(&self.db)
                .await?
        } else {
            vec![]
        };

        let user_map: std::collections::HashMap<Uuid, (String, String)> = users
            .into_iter()
            .map(|(id, name, email)| (id, (name, email)))
            .collect();

        let result: Vec<TaskWithAssignee> = tasks
            .into_iter()
            .map(|task| {
                let (assignee_name, assignee_email) = task
                    .assignee_id
                    .and_then(|id| user_map.get(&id).cloned())
                    .map(|(n, e)| (Some(n), Some(e)))
                    .unwrap_or((None, None));
                TaskWithAssignee {
                    task,
                    assignee_name,
                    assignee_email,
                    created_by_name: None,
                }
            })
            .collect();

        Ok(result)
    }

    // ==================== Task Comments ====================

    /// List comments for a task
    pub async fn list_comments(
        &self,
        org_id: Uuid,
        task_id: Uuid,
    ) -> AppResult<Vec<TaskCommentWithUser>> {
        // Verify task exists and belongs to org
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM tasks WHERE id = $1 AND organization_id = $2",
        )
        .bind(task_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        let comments = sqlx::query_as::<_, TaskComment>(
            r#"
            SELECT id, task_id, user_id, content, created_at
            FROM task_comments
            WHERE task_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.db)
        .await?;

        // Get user info
        let user_ids: Vec<Uuid> = comments.iter().map(|c| c.user_id).collect();

        let users: Vec<(Uuid, String, String)> = if !user_ids.is_empty() {
            sqlx::query_as("SELECT id, name, email FROM users WHERE id = ANY($1)")
                .bind(&user_ids)
                .fetch_all(&self.db)
                .await?
        } else {
            vec![]
        };

        let user_map: std::collections::HashMap<Uuid, (String, String)> = users
            .into_iter()
            .map(|(id, name, email)| (id, (name, email)))
            .collect();

        let result: Vec<TaskCommentWithUser> = comments
            .into_iter()
            .map(|comment| {
                let (user_name, user_email) = user_map
                    .get(&comment.user_id)
                    .cloned()
                    .map(|(n, e)| (Some(n), Some(e)))
                    .unwrap_or((None, None));
                TaskCommentWithUser {
                    comment,
                    user_name,
                    user_email,
                }
            })
            .collect();

        Ok(result)
    }

    /// Add a comment to a task
    pub async fn add_comment(
        &self,
        org_id: Uuid,
        task_id: Uuid,
        user_id: Uuid,
        input: CreateTaskComment,
    ) -> AppResult<TaskComment> {
        // Verify task exists and belongs to org
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM tasks WHERE id = $1 AND organization_id = $2",
        )
        .bind(task_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        if input.content.trim().is_empty() {
            return Err(AppError::BadRequest("Comment content is required".to_string()));
        }

        let comment = sqlx::query_as::<_, TaskComment>(
            r#"
            INSERT INTO task_comments (task_id, user_id, content)
            VALUES ($1, $2, $3)
            RETURNING id, task_id, user_id, content, created_at
            "#,
        )
        .bind(task_id)
        .bind(user_id)
        .bind(&input.content)
        .fetch_one(&self.db)
        .await?;

        Ok(comment)
    }

    // ==================== Recurring Tasks ====================

    /// Calculate the next occurrence date based on recurrence pattern
    fn calculate_next_occurrence(
        &self,
        current: DateTime<Utc>,
        pattern: &str,
        interval: i32,
        day_of_week: Option<i32>,
        day_of_month: Option<i32>,
        month_of_year: Option<i32>,
    ) -> Option<DateTime<Utc>> {
        let interval = interval.max(1) as i64;

        match pattern {
            "daily" => Some(current + Duration::days(interval)),
            "weekly" => {
                // If day_of_week is specified, find next occurrence on that day
                if let Some(dow) = day_of_week {
                    let target_weekday = match dow {
                        0 => Weekday::Sun,
                        1 => Weekday::Mon,
                        2 => Weekday::Tue,
                        3 => Weekday::Wed,
                        4 => Weekday::Thu,
                        5 => Weekday::Fri,
                        6 => Weekday::Sat,
                        _ => Weekday::Mon,
                    };
                    let current_weekday = current.weekday();
                    let days_until = (target_weekday.num_days_from_sunday() as i64
                        - current_weekday.num_days_from_sunday() as i64
                        + 7) % 7;
                    let days_until = if days_until == 0 { 7 * interval } else { days_until + 7 * (interval - 1) };
                    Some(current + Duration::days(days_until))
                } else {
                    Some(current + Duration::weeks(interval))
                }
            }
            "biweekly" => Some(current + Duration::weeks(2 * interval)),
            "monthly" => {
                let dom = day_of_month.unwrap_or(current.day() as i32) as u32;
                let mut next_month = current.month() + interval as u32;
                let mut year = current.year();
                while next_month > 12 {
                    next_month -= 12;
                    year += 1;
                }
                // Handle months with fewer days
                let days_in_month = NaiveDate::from_ymd_opt(year, next_month, 1)
                    .and_then(|d| d.with_month(next_month + 1))
                    .map(|d| d.pred_opt().unwrap().day())
                    .unwrap_or(28);
                let actual_day = dom.min(days_in_month);
                NaiveDate::from_ymd_opt(year, next_month, actual_day)
                    .and_then(|d| d.and_time(NaiveTime::from_hms_opt(0, 0, 0)?).and_local_timezone(Utc).single())
            }
            "quarterly" => {
                let dom = day_of_month.unwrap_or(current.day() as i32) as u32;
                let mut next_month = current.month() + 3 * interval as u32;
                let mut year = current.year();
                while next_month > 12 {
                    next_month -= 12;
                    year += 1;
                }
                let days_in_month = NaiveDate::from_ymd_opt(year, next_month, 1)
                    .and_then(|d| d.with_month(next_month + 1))
                    .map(|d| d.pred_opt().unwrap().day())
                    .unwrap_or(28);
                let actual_day = dom.min(days_in_month);
                NaiveDate::from_ymd_opt(year, next_month, actual_day)
                    .and_then(|d| d.and_time(NaiveTime::from_hms_opt(0, 0, 0)?).and_local_timezone(Utc).single())
            }
            "yearly" => {
                let moy = month_of_year.unwrap_or(current.month() as i32) as u32;
                let dom = day_of_month.unwrap_or(current.day() as i32) as u32;
                let next_year = current.year() + interval as i32;
                let days_in_month = NaiveDate::from_ymd_opt(next_year, moy, 1)
                    .and_then(|d| d.with_month(moy + 1))
                    .map(|d| d.pred_opt().unwrap().day())
                    .unwrap_or(28);
                let actual_day = dom.min(days_in_month);
                NaiveDate::from_ymd_opt(next_year, moy, actual_day)
                    .and_then(|d| d.and_time(NaiveTime::from_hms_opt(0, 0, 0)?).and_local_timezone(Utc).single())
            }
            _ => None,
        }
    }

    /// Get all recurring task templates for an organization
    pub async fn get_recurring_tasks(&self, org_id: Uuid) -> AppResult<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, organization_id, title, description, task_type,
                   related_entity_type, related_entity_id, assignee_id,
                   due_at, completed_at, status, priority,
                   created_by, created_at, updated_at,
                   is_recurring, recurrence_pattern, recurrence_interval,
                   recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                   recurrence_end_at, recurrence_count, recurrence_occurrences,
                   parent_task_id, next_occurrence_at, last_occurrence_at
            FROM tasks
            WHERE organization_id = $1 AND is_recurring = true
            ORDER BY title ASC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(tasks)
    }

    /// Get tasks that need their recurring occurrence created
    pub async fn get_tasks_needing_occurrence(&self, org_id: Uuid) -> AppResult<Vec<Task>> {
        let now = Utc::now();
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, organization_id, title, description, task_type,
                   related_entity_type, related_entity_id, assignee_id,
                   due_at, completed_at, status, priority,
                   created_by, created_at, updated_at,
                   is_recurring, recurrence_pattern, recurrence_interval,
                   recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                   recurrence_end_at, recurrence_count, recurrence_occurrences,
                   parent_task_id, next_occurrence_at, last_occurrence_at
            FROM tasks
            WHERE organization_id = $1
              AND is_recurring = true
              AND next_occurrence_at IS NOT NULL
              AND next_occurrence_at <= $2
              AND (recurrence_end_at IS NULL OR recurrence_end_at > $2)
              AND (recurrence_count IS NULL OR recurrence_occurrences < recurrence_count)
            "#,
        )
        .bind(org_id)
        .bind(now)
        .fetch_all(&self.db)
        .await?;

        Ok(tasks)
    }

    /// Create a new occurrence of a recurring task
    pub async fn create_occurrence(&self, template: &Task) -> AppResult<Task> {
        // Calculate the due date for this occurrence (same as next_occurrence_at)
        let due_at = template.next_occurrence_at;

        // Create the task occurrence
        let occurrence = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (organization_id, title, description, task_type,
                              related_entity_type, related_entity_id, assignee_id,
                              due_at, priority, created_by, status, parent_task_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'open', $11)
            RETURNING id, organization_id, title, description, task_type,
                      related_entity_type, related_entity_id, assignee_id,
                      due_at, completed_at, status, priority,
                      created_by, created_at, updated_at,
                      is_recurring, recurrence_pattern, recurrence_interval,
                      recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                      recurrence_end_at, recurrence_count, recurrence_occurrences,
                      parent_task_id, next_occurrence_at, last_occurrence_at
            "#,
        )
        .bind(template.organization_id)
        .bind(&template.title)
        .bind(&template.description)
        .bind(&template.task_type)
        .bind(&template.related_entity_type)
        .bind(template.related_entity_id)
        .bind(template.assignee_id)
        .bind(due_at)
        .bind(&template.priority)
        .bind(template.created_by)
        .bind(template.id)
        .fetch_one(&self.db)
        .await?;

        // Calculate the next occurrence date
        let next_occurrence = if let Some(pattern) = &template.recurrence_pattern {
            if let Some(current) = template.next_occurrence_at {
                self.calculate_next_occurrence(
                    current,
                    pattern,
                    template.recurrence_interval.unwrap_or(1),
                    template.recurrence_day_of_week,
                    template.recurrence_day_of_month,
                    template.recurrence_month_of_year,
                )
            } else {
                None
            }
        } else {
            None
        };

        // Update the template with new next_occurrence_at and increment occurrences
        sqlx::query(
            r#"
            UPDATE tasks SET
                next_occurrence_at = $2,
                last_occurrence_at = NOW(),
                recurrence_occurrences = COALESCE(recurrence_occurrences, 0) + 1,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(template.id)
        .bind(next_occurrence)
        .execute(&self.db)
        .await?;

        // Record in history
        let occurrence_num = template.recurrence_occurrences.unwrap_or(0) + 1;
        sqlx::query(
            r#"
            INSERT INTO task_recurrence_history (task_id, occurrence_number, created_task_id, scheduled_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(template.id)
        .bind(occurrence_num)
        .bind(occurrence.id)
        .bind(due_at)
        .execute(&self.db)
        .await?;

        self.invalidate_caches(template.organization_id).await;
        Ok(occurrence)
    }

    /// Process all recurring tasks for an organization - creates new occurrences as needed
    pub async fn process_recurring_tasks(&self, org_id: Uuid) -> AppResult<i32> {
        let tasks = self.get_tasks_needing_occurrence(org_id).await?;
        let mut created_count = 0;

        for task in tasks {
            match self.create_occurrence(&task).await {
                Ok(_) => created_count += 1,
                Err(e) => {
                    tracing::warn!("Failed to create occurrence for task {}: {}", task.id, e);
                }
            }
        }

        Ok(created_count)
    }

    /// Get recurrence history for a recurring task template
    pub async fn get_recurrence_history(
        &self,
        org_id: Uuid,
        task_id: Uuid,
    ) -> AppResult<Vec<TaskRecurrenceHistory>> {
        // Verify task exists and belongs to org
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM tasks WHERE id = $1 AND organization_id = $2 AND is_recurring = true",
        )
        .bind(task_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recurring task not found".to_string()))?;

        let history = sqlx::query_as::<_, TaskRecurrenceHistory>(
            r#"
            SELECT id, task_id, occurrence_number, created_task_id, scheduled_at,
                   created_at, skipped, skip_reason
            FROM task_recurrence_history
            WHERE task_id = $1
            ORDER BY occurrence_number DESC
            LIMIT 100
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.db)
        .await?;

        Ok(history)
    }

    /// Get task occurrences for a recurring task template
    pub async fn get_task_occurrences(
        &self,
        org_id: Uuid,
        parent_task_id: Uuid,
    ) -> AppResult<Vec<TaskWithAssignee>> {
        // Verify parent task exists
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM tasks WHERE id = $1 AND organization_id = $2 AND is_recurring = true",
        )
        .bind(parent_task_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recurring task not found".to_string()))?;

        let query = ListTasksQuery {
            parent_task_id: Some(parent_task_id),
            limit: Some(100),
            ..Default::default()
        };

        self.list_tasks(org_id, query).await
    }

    /// Skip the next occurrence of a recurring task
    pub async fn skip_next_occurrence(
        &self,
        org_id: Uuid,
        task_id: Uuid,
        reason: Option<String>,
    ) -> AppResult<Task> {
        // Get the task
        let task = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, organization_id, title, description, task_type,
                   related_entity_type, related_entity_id, assignee_id,
                   due_at, completed_at, status, priority,
                   created_by, created_at, updated_at,
                   is_recurring, recurrence_pattern, recurrence_interval,
                   recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                   recurrence_end_at, recurrence_count, recurrence_occurrences,
                   parent_task_id, next_occurrence_at, last_occurrence_at
            FROM tasks
            WHERE id = $1 AND organization_id = $2 AND is_recurring = true
            "#,
        )
        .bind(task_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recurring task not found".to_string()))?;

        // Calculate the next occurrence
        let next_occurrence = if let (Some(pattern), Some(current)) = (&task.recurrence_pattern, task.next_occurrence_at) {
            self.calculate_next_occurrence(
                current,
                pattern,
                task.recurrence_interval.unwrap_or(1),
                task.recurrence_day_of_week,
                task.recurrence_day_of_month,
                task.recurrence_month_of_year,
            )
        } else {
            None
        };

        // Record the skip in history
        let occurrence_num = task.recurrence_occurrences.unwrap_or(0) + 1;
        sqlx::query(
            r#"
            INSERT INTO task_recurrence_history (task_id, occurrence_number, scheduled_at, skipped, skip_reason)
            VALUES ($1, $2, $3, true, $4)
            "#,
        )
        .bind(task_id)
        .bind(occurrence_num)
        .bind(task.next_occurrence_at)
        .bind(&reason)
        .execute(&self.db)
        .await?;

        // Update the task
        let updated = sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks SET
                next_occurrence_at = $2,
                recurrence_occurrences = COALESCE(recurrence_occurrences, 0) + 1,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, organization_id, title, description, task_type,
                      related_entity_type, related_entity_id, assignee_id,
                      due_at, completed_at, status, priority,
                      created_by, created_at, updated_at,
                      is_recurring, recurrence_pattern, recurrence_interval,
                      recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                      recurrence_end_at, recurrence_count, recurrence_occurrences,
                      parent_task_id, next_occurrence_at, last_occurrence_at
            "#,
        )
        .bind(task_id)
        .bind(next_occurrence)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_caches(org_id).await;
        Ok(updated)
    }

    /// Pause a recurring task (sets next_occurrence_at to NULL)
    pub async fn pause_recurring_task(&self, org_id: Uuid, task_id: Uuid) -> AppResult<Task> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks SET
                next_occurrence_at = NULL,
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND is_recurring = true
            RETURNING id, organization_id, title, description, task_type,
                      related_entity_type, related_entity_id, assignee_id,
                      due_at, completed_at, status, priority,
                      created_by, created_at, updated_at,
                      is_recurring, recurrence_pattern, recurrence_interval,
                      recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                      recurrence_end_at, recurrence_count, recurrence_occurrences,
                      parent_task_id, next_occurrence_at, last_occurrence_at
            "#,
        )
        .bind(task_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recurring task not found".to_string()))?;

        self.invalidate_caches(org_id).await;
        Ok(task)
    }

    /// Resume a paused recurring task
    pub async fn resume_recurring_task(
        &self,
        org_id: Uuid,
        task_id: Uuid,
        resume_from: Option<DateTime<Utc>>,
    ) -> AppResult<Task> {
        let next_at = resume_from.unwrap_or_else(Utc::now);

        let task = sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks SET
                next_occurrence_at = $3,
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND is_recurring = true
            RETURNING id, organization_id, title, description, task_type,
                      related_entity_type, related_entity_id, assignee_id,
                      due_at, completed_at, status, priority,
                      created_by, created_at, updated_at,
                      is_recurring, recurrence_pattern, recurrence_interval,
                      recurrence_day_of_week, recurrence_day_of_month, recurrence_month_of_year,
                      recurrence_end_at, recurrence_count, recurrence_occurrences,
                      parent_task_id, next_occurrence_at, last_occurrence_at
            "#,
        )
        .bind(task_id)
        .bind(org_id)
        .bind(next_at)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Recurring task not found".to_string()))?;

        self.invalidate_caches(org_id).await;
        Ok(task)
    }
}

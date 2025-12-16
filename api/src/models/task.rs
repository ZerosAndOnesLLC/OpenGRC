use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Task type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    ControlTest,
    EvidenceCollection,
    Review,
    Remediation,
    General,
}

impl Default for TaskType {
    fn default() -> Self {
        Self::General
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ControlTest => write!(f, "control_test"),
            Self::EvidenceCollection => write!(f, "evidence_collection"),
            Self::Review => write!(f, "review"),
            Self::Remediation => write!(f, "remediation"),
            Self::General => write!(f, "general"),
        }
    }
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Open,
    InProgress,
    Completed,
    Overdue,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Open
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Overdue => write!(f, "overdue"),
        }
    }
}

/// Task priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Medium
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Recurrence pattern enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecurrencePattern {
    Daily,
    Weekly,
    Biweekly,
    Monthly,
    Quarterly,
    Yearly,
}

impl std::fmt::Display for RecurrencePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Biweekly => write!(f, "biweekly"),
            Self::Monthly => write!(f, "monthly"),
            Self::Quarterly => write!(f, "quarterly"),
            Self::Yearly => write!(f, "yearly"),
        }
    }
}

impl RecurrencePattern {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(Self::Daily),
            "weekly" => Some(Self::Weekly),
            "biweekly" => Some(Self::Biweekly),
            "monthly" => Some(Self::Monthly),
            "quarterly" => Some(Self::Quarterly),
            "yearly" => Some(Self::Yearly),
            _ => None,
        }
    }
}

/// Task entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub task_type: String,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub assignee_id: Option<Uuid>,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub priority: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Recurrence fields
    pub is_recurring: bool,
    pub recurrence_pattern: Option<String>,
    pub recurrence_interval: Option<i32>,
    pub recurrence_day_of_week: Option<i32>,
    pub recurrence_day_of_month: Option<i32>,
    pub recurrence_month_of_year: Option<i32>,
    pub recurrence_end_at: Option<DateTime<Utc>>,
    pub recurrence_count: Option<i32>,
    pub recurrence_occurrences: Option<i32>,
    pub parent_task_id: Option<Uuid>,
    pub next_occurrence_at: Option<DateTime<Utc>>,
    pub last_occurrence_at: Option<DateTime<Utc>>,
}

/// Task with assignee info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskWithAssignee {
    #[serde(flatten)]
    pub task: Task,
    pub assignee_name: Option<String>,
    pub assignee_email: Option<String>,
    pub created_by_name: Option<String>,
}

/// Task comment
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskComment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Task comment with user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCommentWithUser {
    #[serde(flatten)]
    pub comment: TaskComment,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
}

/// Create task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub assignee_id: Option<Uuid>,
    pub due_at: Option<DateTime<Utc>>,
    pub priority: Option<String>,
    // Recurrence fields
    pub is_recurring: Option<bool>,
    pub recurrence_pattern: Option<String>,
    pub recurrence_interval: Option<i32>,
    pub recurrence_day_of_week: Option<i32>,
    pub recurrence_day_of_month: Option<i32>,
    pub recurrence_month_of_year: Option<i32>,
    pub recurrence_end_at: Option<DateTime<Utc>>,
    pub recurrence_count: Option<i32>,
}

/// Update task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub assignee_id: Option<Uuid>,
    pub due_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub priority: Option<String>,
    // Recurrence fields
    pub is_recurring: Option<bool>,
    pub recurrence_pattern: Option<String>,
    pub recurrence_interval: Option<i32>,
    pub recurrence_day_of_week: Option<i32>,
    pub recurrence_day_of_month: Option<i32>,
    pub recurrence_month_of_year: Option<i32>,
    pub recurrence_end_at: Option<DateTime<Utc>>,
    pub recurrence_count: Option<i32>,
}

/// Create task comment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskComment {
    pub content: String,
}

/// List tasks query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTasksQuery {
    pub status: Option<String>,
    pub task_type: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub search: Option<String>,
    pub overdue_only: Option<bool>,
    pub recurring_only: Option<bool>,
    pub exclude_recurring_templates: Option<bool>,
    pub parent_task_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Task statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub total: i64,
    pub open: i64,
    pub in_progress: i64,
    pub completed: i64,
    pub overdue: i64,
    pub by_type: Vec<TaskTypeCount>,
    pub by_priority: Vec<TaskPriorityCount>,
    pub by_assignee: Vec<TaskAssigneeCount>,
    pub due_today: i64,
    pub due_this_week: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskTypeCount {
    pub task_type: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskPriorityCount {
    pub priority: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskAssigneeCount {
    pub assignee_id: Option<Uuid>,
    pub assignee_name: Option<String>,
    pub count: i64,
}

/// Task recurrence history entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskRecurrenceHistory {
    pub id: Uuid,
    pub task_id: Uuid,
    pub occurrence_number: i32,
    pub created_task_id: Option<Uuid>,
    pub scheduled_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub skipped: bool,
    pub skip_reason: Option<String>,
}

impl Task {
    pub fn validate_create(input: &CreateTask) -> Result<(), String> {
        if input.title.trim().is_empty() {
            return Err("Task title is required".to_string());
        }
        if input.title.len() > 500 {
            return Err("Task title must be 500 characters or less".to_string());
        }
        if let Some(ref task_type) = input.task_type {
            if !["control_test", "evidence_collection", "review", "remediation", "general"]
                .contains(&task_type.as_str())
            {
                return Err("Invalid task type".to_string());
            }
        }
        if let Some(ref priority) = input.priority {
            if !["low", "medium", "high", "critical"].contains(&priority.as_str()) {
                return Err("Invalid task priority".to_string());
            }
        }
        // Validate recurrence settings
        if input.is_recurring == Some(true) {
            if input.recurrence_pattern.is_none() {
                return Err("Recurrence pattern is required for recurring tasks".to_string());
            }
            if let Some(ref pattern) = input.recurrence_pattern {
                if !["daily", "weekly", "biweekly", "monthly", "quarterly", "yearly"]
                    .contains(&pattern.as_str())
                {
                    return Err("Invalid recurrence pattern".to_string());
                }
            }
            if let Some(interval) = input.recurrence_interval {
                if interval < 1 {
                    return Err("Recurrence interval must be at least 1".to_string());
                }
            }
            if let Some(dow) = input.recurrence_day_of_week {
                if !(0..=6).contains(&dow) {
                    return Err("Day of week must be 0-6 (Sunday-Saturday)".to_string());
                }
            }
            if let Some(dom) = input.recurrence_day_of_month {
                if !(1..=31).contains(&dom) {
                    return Err("Day of month must be 1-31".to_string());
                }
            }
            if let Some(moy) = input.recurrence_month_of_year {
                if !(1..=12).contains(&moy) {
                    return Err("Month of year must be 1-12".to_string());
                }
            }
        }
        Ok(())
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due_at) = self.due_at {
            self.status != "completed" && due_at < chrono::Utc::now()
        } else {
            false
        }
    }

    pub fn should_create_occurrence(&self, now: DateTime<Utc>) -> bool {
        if !self.is_recurring {
            return false;
        }
        // Check if we've hit the max occurrences
        if let (Some(max), Some(current)) = (self.recurrence_count, self.recurrence_occurrences) {
            if current >= max {
                return false;
            }
        }
        // Check if we're past the end date
        if let Some(end_at) = self.recurrence_end_at {
            if now > end_at {
                return false;
            }
        }
        // Check if the next occurrence is due
        if let Some(next) = self.next_occurrence_at {
            return now >= next;
        }
        false
    }
}

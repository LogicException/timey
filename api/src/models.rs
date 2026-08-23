use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "admin" => Some(Self::Admin),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryStatus {
    Running,
    Complete,
    NeedsTask,
}

impl EntryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Complete => "complete",
            Self::NeedsTask => "needs_task",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "running" => Some(Self::Running),
            "complete" => Some(Self::Complete),
            "needs_task" => Some(Self::NeedsTask),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkSessionStatus {
    Running,
    Paused,
    Stopped,
}

impl WorkSessionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Stopped => "stopped",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "running" => Some(Self::Running),
            "paused" => Some(Self::Paused),
            "stopped" => Some(Self::Stopped),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: Option<String>,
    pub auth_provider: String,
    pub oidc_subject: Option<String>,
    pub role: String,
    pub disabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ProjectRow {
    pub id: i64,
    pub name: String,
    pub archived: bool,
    pub created_by: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct NamedRow {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub archived: bool,
    pub is_system: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct EntryRow {
    pub id: i64,
    pub user_id: i64,
    pub task_id: Option<i64>,
    pub project_id: Option<i64>,
    pub start_at: String,
    pub end_at: Option<String>,
    pub status: String,
    pub created_at: String,
    pub task_name: Option<String>,
    pub project_name: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct WorkSessionRow {
    pub id: i64,
    pub user_id: i64,
    pub local_date: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct WorkEventRow {
    pub id: i64,
    pub work_session_id: i64,
    pub kind: String,
    pub at: String,
}

pub fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(value).map(|dt| dt.with_timezone(&Utc))
}

pub fn parse_date(value: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
}

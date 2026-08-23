use sqlx::{FromRow, SqlitePool};

use crate::domain::default_view::DefaultView;
use crate::domain::working_hours::WorkingHours;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserSettings {
    pub hours: WorkingHours,
    pub default_view: DefaultView,
}

#[derive(Debug, FromRow)]
struct SettingsRow {
    work_start_minutes: i64,
    work_end_minutes: i64,
    default_view: String,
}

impl SettingsRow {
    fn into_settings(self) -> AppResult<UserSettings> {
        let hours = WorkingHours::from_minutes(self.work_start_minutes, self.work_end_minutes)
            .map_err(|err| AppError::Internal(err.message().into()))?;
        let default_view = DefaultView::parse(&self.default_view).ok_or_else(|| {
            AppError::Internal(format!("ungültige Standardansicht: {}", self.default_view))
        })?;
        Ok(UserSettings {
            hours,
            default_view,
        })
    }
}

pub async fn insert_defaults(pool: &SqlitePool, user_id: i64) -> AppResult<()> {
    let hours = WorkingHours::default();
    let default_view = DefaultView::default();
    sqlx::query(
        "INSERT INTO user_settings (user_id, work_start_minutes, work_end_minutes, default_view)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(user_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(hours.start_minutes)
    .bind(hours.end_minutes)
    .bind(default_view.as_str())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_or_default(pool: &SqlitePool, user_id: i64) -> AppResult<UserSettings> {
    let row = sqlx::query_as::<_, SettingsRow>(
        "SELECT work_start_minutes, work_end_minutes, default_view
         FROM user_settings WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        return row.into_settings();
    }

    insert_defaults(pool, user_id).await?;
    Ok(UserSettings {
        hours: WorkingHours::default(),
        default_view: DefaultView::default(),
    })
}

pub async fn update(
    pool: &SqlitePool,
    user_id: i64,
    work_start: &str,
    work_end: &str,
    default_view: Option<&str>,
) -> AppResult<UserSettings> {
    let hours = WorkingHours::parse(work_start, work_end)
        .map_err(|err| AppError::Unprocessable(err.message().into()))?;
    let current = get_or_default(pool, user_id).await?;
    let default_view = match default_view {
        Some(value) => DefaultView::parse(value).ok_or_else(|| {
            AppError::Unprocessable("Standardansicht muss day oder week sein".into())
        })?,
        None => current.default_view,
    };
    sqlx::query(
        "INSERT INTO user_settings (user_id, work_start_minutes, work_end_minutes, default_view)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET
            work_start_minutes = excluded.work_start_minutes,
            work_end_minutes = excluded.work_end_minutes,
            default_view = excluded.default_view",
    )
    .bind(user_id)
    .bind(hours.start_minutes)
    .bind(hours.end_minutes)
    .bind(default_view.as_str())
    .execute(pool)
    .await?;
    Ok(UserSettings {
        hours,
        default_view,
    })
}

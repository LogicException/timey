use sqlx::{FromRow, SqlitePool};

use crate::domain::working_hours::WorkingHours;
use crate::error::{AppError, AppResult};

#[derive(Debug, FromRow)]
struct SettingsRow {
    work_start_minutes: i64,
    work_end_minutes: i64,
}

pub async fn insert_defaults(pool: &SqlitePool, user_id: i64) -> AppResult<()> {
    let hours = WorkingHours::default();
    sqlx::query(
        "INSERT INTO user_settings (user_id, work_start_minutes, work_end_minutes)
         VALUES (?, ?, ?)
         ON CONFLICT(user_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(hours.start_minutes)
    .bind(hours.end_minutes)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_or_default(pool: &SqlitePool, user_id: i64) -> AppResult<WorkingHours> {
    let row = sqlx::query_as::<_, SettingsRow>(
        "SELECT work_start_minutes, work_end_minutes FROM user_settings WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        return WorkingHours::from_minutes(row.work_start_minutes, row.work_end_minutes)
            .map_err(|err| AppError::Internal(err.message().into()));
    }

    insert_defaults(pool, user_id).await?;
    Ok(WorkingHours::default())
}

pub async fn update(
    pool: &SqlitePool,
    user_id: i64,
    work_start: &str,
    work_end: &str,
) -> AppResult<WorkingHours> {
    let hours = WorkingHours::parse(work_start, work_end)
        .map_err(|err| AppError::Unprocessable(err.message().into()))?;
    sqlx::query(
        "INSERT INTO user_settings (user_id, work_start_minutes, work_end_minutes)
         VALUES (?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET
            work_start_minutes = excluded.work_start_minutes,
            work_end_minutes = excluded.work_end_minutes",
    )
    .bind(user_id)
    .bind(hours.start_minutes)
    .bind(hours.end_minutes)
    .execute(pool)
    .await?;
    Ok(hours)
}

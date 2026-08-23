use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::domain::{APP_TZ, close_timestamp, needs_midnight_close};
use crate::error::{AppError, AppResult};
use crate::models::{EntryRow, EntryStatus, parse_rfc3339};
use crate::services::work;

pub async fn close_stale_for_user(
    pool: &SqlitePool,
    user_id: i64,
    now: DateTime<Utc>,
) -> AppResult<()> {
    work::close_if_stale(pool, user_id, now).await?;

    let running = sqlx::query_as::<_, EntryRow>(
        "SELECT e.id, e.user_id, e.task_id, e.project_id, e.start_at, e.end_at, e.status, e.created_at,
                t.name AS task_name, p.name AS project_name
         FROM entries e
         LEFT JOIN tasks t ON t.id = e.task_id
         LEFT JOIN projects p ON p.id = e.project_id
         WHERE e.user_id = ? AND e.status = 'running'",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    for row in running {
        let start = parse_rfc3339(&row.start_at)
            .map_err(|_| AppError::Internal("Startzeit ungültig".into()))?;
        let started_on = crate::domain::civil_date(start, APP_TZ);
        if !needs_midnight_close(started_on, now, APP_TZ) {
            continue;
        }
        let end = close_timestamp(started_on, APP_TZ)
            .ok_or_else(|| AppError::Internal("Mitternachtszeit ungültig".into()))?;
        let status = if row.task_id.is_some() {
            EntryStatus::Complete
        } else {
            EntryStatus::NeedsTask
        };
        sqlx::query("UPDATE entries SET end_at = ?, status = ? WHERE id = ?")
            .bind(end.to_rfc3339())
            .bind(status.as_str())
            .bind(row.id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

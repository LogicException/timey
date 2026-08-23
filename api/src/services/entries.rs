use chrono::{DateTime, NaiveDate, Utc};
use sqlx::SqlitePool;

use crate::domain::{
    APP_TZ, Interval, any_contains_instant, any_overlap, civil_date, same_civil_day,
};
use crate::error::{AppError, AppResult};
use crate::models::{EntryRow, EntryStatus, parse_rfc3339};
use crate::services::catalogs::{get_project, get_task};

#[derive(Debug, Clone)]
pub struct NewEntry {
    pub task_id: Option<i64>,
    pub project_id: Option<i64>,
    pub start_at: DateTime<Utc>,
    pub end_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct EntryFilters {
    pub task_ids: Vec<i64>,
    pub project_ids: Vec<i64>,
}

pub async fn list_entries(
    pool: &SqlitePool,
    user_id: i64,
    from: NaiveDate,
    to: NaiveDate,
    filters: &EntryFilters,
) -> AppResult<Vec<EntryRow>> {
    if to < from {
        return Err(AppError::Unprocessable(
            "Endedatum darf nicht vor dem Startdatum liegen".into(),
        ));
    }
    let start = crate::domain::same_day::start_of_day(from, APP_TZ)
        .ok_or_else(|| AppError::Internal("Start des Tages ungültig".into()))?;
    let end_exclusive_date = to
        .succ_opt()
        .ok_or_else(|| AppError::Internal("Endedatum ungültig".into()))?;
    let end = crate::domain::same_day::start_of_day(end_exclusive_date, APP_TZ)
        .ok_or_else(|| AppError::Internal("Ende des Tages ungültig".into()))?;

    let rows = sqlx::query_as::<_, EntryRow>(
        "SELECT e.id, e.user_id, e.task_id, e.project_id, e.start_at, e.end_at, e.status, e.created_at,
                t.name AS task_name, p.name AS project_name
         FROM entries e
         LEFT JOIN tasks t ON t.id = e.task_id
         LEFT JOIN projects p ON p.id = e.project_id
         WHERE e.user_id = ? AND e.start_at >= ? AND e.start_at < ?
         ORDER BY e.start_at DESC",
    )
    .bind(user_id)
    .bind(start.to_rfc3339())
    .bind(end.to_rfc3339())
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter(|row| passes_filters(row, filters))
        .collect())
}

fn passes_filters(row: &EntryRow, filters: &EntryFilters) -> bool {
    if !filters.task_ids.is_empty() && !row.task_id.is_some_and(|id| filters.task_ids.contains(&id))
    {
        return false;
    }
    if !filters.project_ids.is_empty()
        && !row
            .project_id
            .is_some_and(|id| filters.project_ids.contains(&id))
    {
        return false;
    }
    true
}

pub async fn create_entry(
    pool: &SqlitePool,
    user_id: i64,
    input: NewEntry,
    now: DateTime<Utc>,
) -> AppResult<EntryRow> {
    let (status, end_at) = match input.end_at {
        None => (EntryStatus::Running, None),
        Some(end) => {
            validate_complete(&input, end)?;
            (EntryStatus::Complete, Some(end))
        }
    };

    if status == EntryStatus::Running {
        ensure_no_open_timer(pool, user_id).await?;
        ensure_no_needs_task(pool, user_id).await?;
    }

    validate_refs(pool, user_id, input.task_id, input.project_id).await?;
    ensure_same_day(input.start_at, end_at.unwrap_or(input.start_at))?;
    ensure_no_overlap(pool, user_id, None, input.start_at, end_at, now).await?;

    insert_entry(pool, user_id, &input, end_at, status, now).await
}

pub async fn update_entry(
    pool: &SqlitePool,
    user_id: i64,
    id: i64,
    input: NewEntry,
    now: DateTime<Utc>,
) -> AppResult<EntryRow> {
    let existing = get_owned_entry(pool, user_id, id).await?;
    let end_at = input.end_at;
    let status = if end_at.is_none() {
        EntryStatus::Running
    } else if input.task_id.is_none() {
        EntryStatus::NeedsTask
    } else {
        EntryStatus::Complete
    };

    if status == EntryStatus::Complete {
        let end = end_at
            .ok_or_else(|| AppError::Internal("Abgeschlossener Eintrag ohne Endzeit".into()))?;
        validate_complete(&input, end)?;
    }
    if status == EntryStatus::Running {
        ensure_no_open_timer_except(pool, user_id, id).await?;
        if existing.status != EntryStatus::Running.as_str() {
            ensure_no_needs_task(pool, user_id).await?;
        }
    }

    validate_refs(pool, user_id, input.task_id, input.project_id).await?;
    ensure_same_day(input.start_at, end_at.unwrap_or(input.start_at))?;
    ensure_no_overlap(pool, user_id, Some(id), input.start_at, end_at, now).await?;

    sqlx::query(
        "UPDATE entries SET task_id = ?, project_id = ?, start_at = ?, end_at = ?, status = ?
         WHERE id = ? AND user_id = ?",
    )
    .bind(input.task_id)
    .bind(input.project_id)
    .bind(input.start_at.to_rfc3339())
    .bind(end_at.map(|value| value.to_rfc3339()))
    .bind(status.as_str())
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    get_owned_entry(pool, user_id, id).await
}

pub async fn delete_entry(pool: &SqlitePool, user_id: i64, id: i64) -> AppResult<()> {
    let done = sqlx::query("DELETE FROM entries WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    if done.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn start_timer(
    pool: &SqlitePool,
    user_id: i64,
    now: DateTime<Utc>,
) -> AppResult<EntryRow> {
    create_entry(
        pool,
        user_id,
        NewEntry {
            task_id: None,
            project_id: None,
            start_at: now,
            end_at: None,
        },
        now,
    )
    .await
}

pub async fn stop_timer(
    pool: &SqlitePool,
    user_id: i64,
    task_id: i64,
    project_id: Option<i64>,
    now: DateTime<Utc>,
) -> AppResult<EntryRow> {
    let running = running_entry(pool, user_id)
        .await?
        .ok_or_else(|| AppError::Unprocessable("Kein laufender Eintrag".into()))?;
    let start = parse_rfc3339(&running.start_at)
        .map_err(|_| AppError::Internal("Startzeit ungültig".into()))?;
    update_entry(
        pool,
        user_id,
        running.id,
        NewEntry {
            task_id: Some(task_id),
            project_id,
            start_at: start,
            end_at: Some(now),
        },
        now,
    )
    .await
}

pub async fn running_entry(pool: &SqlitePool, user_id: i64) -> AppResult<Option<EntryRow>> {
    let sql = entry_select_sql("e.user_id = ? AND e.status = 'running'");
    Ok(sqlx::query_as::<_, EntryRow>(&sql)
        .bind(user_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_owned_entry(pool: &SqlitePool, user_id: i64, id: i64) -> AppResult<EntryRow> {
    let sql = entry_select_sql("e.id = ? AND e.user_id = ?");
    sqlx::query_as::<_, EntryRow>(&sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn assign_task_to_needs_task(
    pool: &SqlitePool,
    user_id: i64,
    id: i64,
    task_id: i64,
    project_id: Option<i64>,
    now: DateTime<Utc>,
) -> AppResult<EntryRow> {
    let existing = get_owned_entry(pool, user_id, id).await?;
    if existing.status != EntryStatus::NeedsTask.as_str() {
        return Err(AppError::Unprocessable(
            "Eintrag erwartet keinen nachträglichen Task".into(),
        ));
    }
    let start = parse_rfc3339(&existing.start_at)
        .map_err(|_| AppError::Internal("Startzeit ungültig".into()))?;
    let end = existing
        .end_at
        .as_deref()
        .ok_or_else(|| AppError::Internal("needs_task ohne Ende".into()))?;
    let end = parse_rfc3339(end).map_err(|_| AppError::Internal("Endzeit ungültig".into()))?;
    update_entry(
        pool,
        user_id,
        id,
        NewEntry {
            task_id: Some(task_id),
            project_id,
            start_at: start,
            end_at: Some(end),
        },
        now,
    )
    .await
}

async fn insert_entry(
    pool: &SqlitePool,
    user_id: i64,
    input: &NewEntry,
    end_at: Option<DateTime<Utc>>,
    status: EntryStatus,
    now: DateTime<Utc>,
) -> AppResult<EntryRow> {
    let done = sqlx::query(
        "INSERT INTO entries (user_id, task_id, project_id, start_at, end_at, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(input.task_id)
    .bind(input.project_id)
    .bind(input.start_at.to_rfc3339())
    .bind(end_at.map(|value| value.to_rfc3339()))
    .bind(status.as_str())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;
    get_owned_entry(pool, user_id, done.last_insert_rowid()).await
}

fn validate_complete(input: &NewEntry, end: DateTime<Utc>) -> AppResult<()> {
    if input.task_id.is_none() {
        return Err(AppError::Unprocessable(
            "Ein abgeschlossener Eintrag braucht einen Task".into(),
        ));
    }
    if end < input.start_at {
        return Err(AppError::Unprocessable(
            "Endzeitpunkt darf nicht vor dem Start liegen".into(),
        ));
    }
    if end == input.start_at {
        return Err(AppError::Unprocessable(
            "Eintrag braucht eine Dauer größer als 0".into(),
        ));
    }
    Ok(())
}

fn ensure_same_day(start: DateTime<Utc>, end: DateTime<Utc>) -> AppResult<()> {
    if !same_civil_day(start, end, APP_TZ) {
        return Err(AppError::Unprocessable(
            "Start und Ende müssen am selben Kalendertag liegen".into(),
        ));
    }
    Ok(())
}

async fn validate_refs(
    pool: &SqlitePool,
    user_id: i64,
    task_id: Option<i64>,
    project_id: Option<i64>,
) -> AppResult<()> {
    if let Some(task_id) = task_id {
        let task = get_task(pool, user_id, task_id).await?;
        if task.archived {
            return Err(AppError::Unprocessable("Task ist archiviert".into()));
        }
    }
    if let Some(project_id) = project_id {
        let project = get_project(pool, project_id).await?;
        if project.archived {
            return Err(AppError::Unprocessable("Projekt ist archiviert".into()));
        }
    }
    Ok(())
}

async fn ensure_no_open_timer(pool: &SqlitePool, user_id: i64) -> AppResult<()> {
    if running_entry(pool, user_id).await?.is_some() {
        return Err(AppError::Conflict("Es läuft bereits ein Eintrag".into()));
    }
    Ok(())
}

async fn ensure_no_open_timer_except(pool: &SqlitePool, user_id: i64, id: i64) -> AppResult<()> {
    if let Some(running) = running_entry(pool, user_id).await?
        && running.id != id
    {
        return Err(AppError::Conflict("Es läuft bereits ein Eintrag".into()));
    }
    Ok(())
}

pub async fn ensure_no_needs_task(pool: &SqlitePool, user_id: i64) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM entries WHERE user_id = ? AND status = 'needs_task'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if count > 0 {
        return Err(AppError::Unprocessable(
            "Offene Einträge ohne Task zuerst zuordnen".into(),
        ));
    }
    Ok(())
}

async fn ensure_no_overlap(
    pool: &SqlitePool,
    user_id: i64,
    except_id: Option<i64>,
    start: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> AppResult<()> {
    let candidate = match end {
        Some(end) => Interval::new(start, end)
            .ok_or_else(|| AppError::Unprocessable("Zeitraum ungültig".into()))?,
        None => Interval::running(start, now),
    };

    let sql = entry_select_sql("e.user_id = ?");
    let rows = sqlx::query_as::<_, EntryRow>(&sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    let mut existing = Vec::new();
    for row in rows {
        if except_id == Some(row.id) {
            continue;
        }
        let row_start = parse_rfc3339(&row.start_at)
            .map_err(|_| AppError::Internal("Startzeit ungültig".into()))?;
        let interval = match row.end_at.as_deref() {
            Some(end) => {
                let end = parse_rfc3339(end)
                    .map_err(|_| AppError::Internal("Endzeit ungültig".into()))?;
                Interval::new(row_start, end)
                    .ok_or_else(|| AppError::Internal("Gespeicherter Zeitraum ungültig".into()))?
            }
            None => Interval::running(row_start, now),
        };
        existing.push(interval);
    }

    if any_overlap(candidate, &existing)
        || (end.is_none() && any_contains_instant(start, &existing))
    {
        return Err(AppError::Conflict(
            "Eintrag überschneidet sich mit einem bestehenden Zeitraum".into(),
        ));
    }
    Ok(())
}

fn entry_select_sql(where_clause: &str) -> String {
    format!(
        "SELECT e.id, e.user_id, e.task_id, e.project_id, e.start_at, e.end_at, e.status, e.created_at,
                t.name AS task_name, p.name AS project_name
         FROM entries e
         LEFT JOIN tasks t ON t.id = e.task_id
         LEFT JOIN projects p ON p.id = e.project_id
         WHERE {where_clause}"
    )
}

pub fn format_hm(total_seconds: i64) -> String {
    let seconds = total_seconds.max(0);
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    format!("{hours}:{minutes:02}")
}

pub fn format_duration_label(start: DateTime<Utc>, end: DateTime<Utc>) -> String {
    format_hm((end - start).num_seconds())
}

pub fn csv_escape(field: &str) -> String {
    if field.contains(['"', ',', '\n', '\r', ';']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

pub fn entries_to_csv(rows: &[EntryRow]) -> AppResult<String> {
    let mut out = String::from('\u{feff}');
    out.push_str("Start,Ende,Dauer,Task,Projekt\n");
    let mut total_seconds: i64 = 0;
    for row in rows {
        let start = parse_rfc3339(&row.start_at)
            .map_err(|_| AppError::Internal("Startzeit ungültig".into()))?;
        let end = row
            .end_at
            .as_deref()
            .map(parse_rfc3339)
            .transpose()
            .map_err(|_| AppError::Internal("Endzeit ungültig".into()))?;
        if let Some(end) = end {
            total_seconds += (end - start).num_seconds().max(0);
        }
        let duration = end
            .map(|end| format_duration_label(start, end))
            .unwrap_or_default();
        let berlin_start = start
            .with_timezone(&APP_TZ)
            .format("%Y-%m-%d %H:%M")
            .to_string();
        let berlin_end = end
            .map(|end| {
                end.with_timezone(&APP_TZ)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_default();
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            csv_escape(&berlin_start),
            csv_escape(&berlin_end),
            csv_escape(&duration),
            csv_escape(row.task_name.as_deref().unwrap_or("")),
            csv_escape(row.project_name.as_deref().unwrap_or("")),
        ));
    }
    out.push_str(&format!(
        "{},,{},,\n",
        csv_escape("Summe"),
        csv_escape(&format_hm(total_seconds)),
    ));
    Ok(out)
}

pub fn today(now: DateTime<Utc>) -> NaiveDate {
    civil_date(now, APP_TZ)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn csv_escape_quotes_comma_fields() {
        assert_eq!(csv_escape("Kunde, XYZ"), "\"Kunde, XYZ\"");
        assert_eq!(csv_escape("plain"), "plain");
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn duration_label_formats_hours_and_minutes() {
        let start = Utc::now();
        let end = start + Duration::minutes(90);
        assert_eq!(format_duration_label(start, end), "1:30");
    }

    fn entry(
        start_at: &str,
        end_at: Option<&str>,
        task_name: Option<&str>,
        project_name: Option<&str>,
    ) -> EntryRow {
        EntryRow {
            id: 1,
            user_id: 1,
            task_id: None,
            project_id: None,
            start_at: start_at.to_string(),
            end_at: end_at.map(str::to_string),
            status: "complete".into(),
            created_at: start_at.to_string(),
            task_name: task_name.map(str::to_string),
            project_name: project_name.map(str::to_string),
        }
    }

    #[test]
    fn csv_empty_list_includes_zero_sum_row() {
        let csv = entries_to_csv(&[]).expect("csv");
        assert!(
            csv.contains("Start,Ende,Dauer,Task,Projekt\n"),
            "expected header without Aufgabe, got {csv:?}"
        );
        assert!(
            !csv.contains("Aufgabe"),
            "CSV must not contain Aufgabe, got {csv:?}"
        );
        assert!(
            csv.ends_with("Summe,,0:00,,\n"),
            "expected sum footer, got {csv:?}"
        );
    }

    #[test]
    fn csv_sums_completed_entries() {
        let csv = entries_to_csv(&[
            entry(
                "2026-08-21T12:00:00Z",
                Some("2026-08-21T12:30:00Z"),
                Some("Meeting"),
                Some("Elba"),
            ),
            entry(
                "2026-08-21T09:45:00Z",
                Some("2026-08-21T11:00:00Z"),
                Some("E-Mail schreiben"),
                Some("Efa"),
            ),
        ])
        .expect("csv");
        assert!(
            csv.contains("Summe,,1:45,,\n"),
            "expected 1:45 sum, got {csv:?}"
        );
    }

    #[test]
    fn csv_running_entry_does_not_increase_sum() {
        let csv = entries_to_csv(&[
            entry(
                "2026-08-21T12:00:00Z",
                Some("2026-08-21T12:30:00Z"),
                Some("Meeting"),
                None,
            ),
            entry("2026-08-21T13:00:00Z", None, Some("läuft"), None),
        ])
        .expect("csv");
        assert!(
            csv.contains("Summe,,0:30,,\n"),
            "expected 0:30 sum, got {csv:?}"
        );
    }
}

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::SqlitePool;

use crate::domain::{APP_TZ, Interval, WorkEvent, WorkEventKind, running_intervals};
use crate::error::{AppError, AppResult};
use crate::models::{WorkEventRow, WorkSessionRow, WorkSessionStatus, parse_date, parse_rfc3339};
use crate::services::entries::today;

pub struct WorkSnapshot {
    pub session: Option<WorkSessionRow>,
    pub elapsed_seconds: i64,
    pub local_date: NaiveDate,
}

pub struct WorkDaySummary {
    pub local_date: NaiveDate,
    pub elapsed_seconds: i64,
    pub intervals: Vec<Interval>,
}

pub async fn current(
    pool: &SqlitePool,
    user_id: i64,
    now: DateTime<Utc>,
) -> AppResult<WorkSnapshot> {
    let local_date = today(now);
    let sessions = sessions_on(pool, user_id, local_date).await?;
    let open = sessions
        .iter()
        .find(|row| row.status != WorkSessionStatus::Stopped.as_str())
        .cloned();

    let elapsed_seconds = elapsed_on(pool, user_id, local_date, now).await?;

    Ok(WorkSnapshot {
        session: open,
        elapsed_seconds,
        local_date,
    })
}

pub async fn list_for_range(
    pool: &SqlitePool,
    user_id: i64,
    from: NaiveDate,
    to: NaiveDate,
    now: DateTime<Utc>,
) -> AppResult<Vec<WorkDaySummary>> {
    if to < from {
        return Err(AppError::Unprocessable(
            "Endedatum darf nicht vor dem Startdatum liegen".into(),
        ));
    }

    let dates = session_dates_in(pool, user_id, from, to).await?;
    let mut days = Vec::with_capacity(dates.len());
    for date in dates {
        let intervals = intervals_on(pool, user_id, date, now).await?;
        days.push(WorkDaySummary {
            local_date: date,
            elapsed_seconds: elapsed_from(&intervals),
            intervals,
        });
    }
    Ok(days)
}

pub async fn elapsed_on(
    pool: &SqlitePool,
    user_id: i64,
    local_date: NaiveDate,
    now: DateTime<Utc>,
) -> AppResult<i64> {
    let intervals = intervals_on(pool, user_id, local_date, now).await?;
    Ok(elapsed_from(&intervals))
}

fn elapsed_from(intervals: &[Interval]) -> i64 {
    intervals
        .iter()
        .map(|interval| (interval.end - interval.start).num_seconds())
        .sum::<i64>()
        .max(0)
}

async fn intervals_on(
    pool: &SqlitePool,
    user_id: i64,
    local_date: NaiveDate,
    now: DateTime<Utc>,
) -> AppResult<Vec<Interval>> {
    let sessions = sessions_on(pool, user_id, local_date).await?;
    let mut intervals = Vec::new();
    for session in &sessions {
        let events = events_for(pool, session.id).await?;
        intervals.extend(running_intervals(&events, now));
    }
    Ok(intervals)
}

pub async fn start(pool: &SqlitePool, user_id: i64, now: DateTime<Utc>) -> AppResult<WorkSnapshot> {
    let local_date = today(now);
    if let Some(open) = open_session(pool, user_id, local_date).await? {
        let status = WorkSessionStatus::parse(&open.status);
        return match status {
            Some(WorkSessionStatus::Running) => {
                Err(AppError::Conflict("Arbeitszeit läuft bereits".into()))
            }
            Some(WorkSessionStatus::Paused) => Err(AppError::Conflict(
                "Arbeitszeit ist pausiert — fortsetzen statt starten".into(),
            )),
            _ => Err(AppError::Conflict(
                "Offene Arbeitszeit-Session vorhanden".into(),
            )),
        };
    }

    let done = sqlx::query(
        "INSERT INTO work_sessions (user_id, local_date, status, created_at) VALUES (?, ?, 'running', ?)",
    )
    .bind(user_id)
    .bind(local_date.format("%Y-%m-%d").to_string())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;
    insert_event(pool, done.last_insert_rowid(), WorkEventKind::Started, now).await?;
    current(pool, user_id, now).await
}

pub async fn require_running(pool: &SqlitePool, user_id: i64, now: DateTime<Utc>) -> AppResult<()> {
    let local_date = today(now);
    let session = open_session(pool, user_id, local_date).await?;
    match session
        .as_ref()
        .and_then(|row| WorkSessionStatus::parse(&row.status))
    {
        Some(WorkSessionStatus::Running) => Ok(()),
        _ => Err(AppError::Unprocessable("Arbeitszeit läuft nicht".into())),
    }
}

pub async fn pause(pool: &SqlitePool, user_id: i64, now: DateTime<Utc>) -> AppResult<WorkSnapshot> {
    reject_if_entry_running(pool, user_id).await?;
    let session = require_status(pool, user_id, now, WorkSessionStatus::Running).await?;
    set_status(pool, session.id, WorkSessionStatus::Paused).await?;
    insert_event(pool, session.id, WorkEventKind::Paused, now).await?;
    current(pool, user_id, now).await
}

pub async fn resume(
    pool: &SqlitePool,
    user_id: i64,
    now: DateTime<Utc>,
) -> AppResult<WorkSnapshot> {
    let session = require_status(pool, user_id, now, WorkSessionStatus::Paused).await?;
    set_status(pool, session.id, WorkSessionStatus::Running).await?;
    insert_event(pool, session.id, WorkEventKind::Resumed, now).await?;
    current(pool, user_id, now).await
}

pub async fn stop(pool: &SqlitePool, user_id: i64, now: DateTime<Utc>) -> AppResult<WorkSnapshot> {
    reject_if_entry_running(pool, user_id).await?;
    let local_date = today(now);
    let session = open_session(pool, user_id, local_date)
        .await?
        .ok_or_else(|| AppError::Unprocessable("Keine laufende Arbeitszeit".into()))?;
    set_status(pool, session.id, WorkSessionStatus::Stopped).await?;
    insert_event(pool, session.id, WorkEventKind::Stopped, now).await?;
    current(pool, user_id, now).await
}

pub async fn close_if_stale(pool: &SqlitePool, user_id: i64, now: DateTime<Utc>) -> AppResult<()> {
    let rows = sqlx::query_as::<_, WorkSessionRow>(
        "SELECT id, user_id, local_date, status, created_at FROM work_sessions
         WHERE user_id = ? AND status != 'stopped'",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    for row in rows {
        let started_on = parse_date(&row.local_date)
            .map_err(|_| AppError::Internal("local_date ungültig".into()))?;
        if !crate::domain::needs_midnight_close(started_on, now, APP_TZ) {
            continue;
        }
        let close_at = crate::domain::close_timestamp(started_on, APP_TZ)
            .ok_or_else(|| AppError::Internal("Mitternachtszeit ungültig".into()))?;
        if row.status == WorkSessionStatus::Running.as_str()
            || row.status == WorkSessionStatus::Paused.as_str()
        {
            insert_event(pool, row.id, WorkEventKind::Stopped, close_at).await?;
        }
        set_status(pool, row.id, WorkSessionStatus::Stopped).await?;
    }
    Ok(())
}

async fn reject_if_entry_running(pool: &SqlitePool, user_id: i64) -> AppResult<()> {
    if crate::services::entries::running_entry(pool, user_id)
        .await?
        .is_some()
    {
        return Err(AppError::Unprocessable(
            "Zuerst den laufenden Eintrag stoppen".into(),
        ));
    }
    Ok(())
}

async fn require_status(
    pool: &SqlitePool,
    user_id: i64,
    now: DateTime<Utc>,
    expected: WorkSessionStatus,
) -> AppResult<WorkSessionRow> {
    let local_date = today(now);
    let session = open_session(pool, user_id, local_date)
        .await?
        .ok_or_else(|| AppError::Unprocessable("Keine offene Arbeitszeit".into()))?;
    if session.status != expected.as_str() {
        return Err(AppError::Unprocessable(format!(
            "Arbeitszeit ist nicht im Zustand {}",
            expected.as_str()
        )));
    }
    Ok(session)
}

async fn open_session(
    pool: &SqlitePool,
    user_id: i64,
    local_date: NaiveDate,
) -> AppResult<Option<WorkSessionRow>> {
    Ok(sqlx::query_as::<_, WorkSessionRow>(
        "SELECT id, user_id, local_date, status, created_at FROM work_sessions
         WHERE user_id = ? AND local_date = ? AND status != 'stopped'
         ORDER BY id DESC",
    )
    .bind(user_id)
    .bind(local_date.format("%Y-%m-%d").to_string())
    .fetch_optional(pool)
    .await?)
}

async fn session_dates_in(
    pool: &SqlitePool,
    user_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<NaiveDate>> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT local_date FROM work_sessions
         WHERE user_id = ? AND local_date >= ? AND local_date <= ?
         ORDER BY local_date",
    )
    .bind(user_id)
    .bind(from.format("%Y-%m-%d").to_string())
    .bind(to.format("%Y-%m-%d").to_string())
    .fetch_all(pool)
    .await?;

    let mut dates = Vec::with_capacity(rows.len());
    for row in rows {
        dates.push(parse_date(&row).map_err(|_| AppError::Internal("local_date ungültig".into()))?);
    }
    Ok(dates)
}

async fn sessions_on(
    pool: &SqlitePool,
    user_id: i64,
    local_date: NaiveDate,
) -> AppResult<Vec<WorkSessionRow>> {
    Ok(sqlx::query_as::<_, WorkSessionRow>(
        "SELECT id, user_id, local_date, status, created_at FROM work_sessions
         WHERE user_id = ? AND local_date = ? ORDER BY id",
    )
    .bind(user_id)
    .bind(local_date.format("%Y-%m-%d").to_string())
    .fetch_all(pool)
    .await?)
}

async fn events_for(pool: &SqlitePool, session_id: i64) -> AppResult<Vec<WorkEvent>> {
    let rows = sqlx::query_as::<_, WorkEventRow>(
        "SELECT id, work_session_id, kind, at FROM work_session_events WHERE work_session_id = ? ORDER BY id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let kind = match row.kind.as_str() {
            "started" => WorkEventKind::Started,
            "paused" => WorkEventKind::Paused,
            "resumed" => WorkEventKind::Resumed,
            "stopped" => WorkEventKind::Stopped,
            _ => return Err(AppError::Internal("Unbekanntes Work-Event".into())),
        };
        let at =
            parse_rfc3339(&row.at).map_err(|_| AppError::Internal("Event-Zeit ungültig".into()))?;
        events.push(WorkEvent { kind, at });
    }
    Ok(events)
}

async fn insert_event(
    pool: &SqlitePool,
    session_id: i64,
    kind: WorkEventKind,
    at: DateTime<Utc>,
) -> AppResult<()> {
    let kind = match kind {
        WorkEventKind::Started => "started",
        WorkEventKind::Paused => "paused",
        WorkEventKind::Resumed => "resumed",
        WorkEventKind::Stopped => "stopped",
    };
    sqlx::query("INSERT INTO work_session_events (work_session_id, kind, at) VALUES (?, ?, ?)")
        .bind(session_id)
        .bind(kind)
        .bind(at.to_rfc3339())
        .execute(pool)
        .await?;
    Ok(())
}

async fn set_status(pool: &SqlitePool, id: i64, status: WorkSessionStatus) -> AppResult<()> {
    sqlx::query("UPDATE work_sessions SET status = ? WHERE id = ?")
        .bind(status.as_str())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

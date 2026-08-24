use chrono::{DateTime, NaiveDate, Utc};
use sqlx::SqlitePool;

use crate::domain::{
    APP_TZ, Interval, LabeledInterval, WorkEventKind, WorkEventRecord, any_overlap, civil_date,
    labeled_intervals, remove_interval, same_civil_day, timestamps_non_decreasing,
    truncate_to_minute,
};
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
    pub intervals: Vec<LabeledInterval>,
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

fn elapsed_from(intervals: &[LabeledInterval]) -> i64 {
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
) -> AppResult<Vec<LabeledInterval>> {
    let sessions = sessions_on(pool, user_id, local_date).await?;
    let mut intervals = Vec::new();
    for session in &sessions {
        let events = event_records_for(pool, session.id).await?;
        intervals.extend(labeled_intervals(session.id, &events, now));
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

    ensure_timer_slot_free(pool, user_id, local_date, now).await?;

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
    let local_date = parse_date(&session.local_date)
        .map_err(|_| AppError::Internal("local_date ungültig".into()))?;
    ensure_timer_slot_free(pool, user_id, local_date, now).await?;
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

pub async fn create_interval(
    pool: &SqlitePool,
    user_id: i64,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> AppResult<LabeledInterval> {
    let (start, end) = validate_closed(start_at, end_at, now, None)?;
    let local_date = civil_date(start, APP_TZ);
    let candidate = Interval::new(start, end)
        .ok_or_else(|| AppError::Unprocessable("Zeitraum ungültig".into()))?;
    ensure_no_work_overlap(pool, user_id, local_date, candidate, None, now).await?;

    let done = sqlx::query(
        "INSERT INTO work_sessions (user_id, local_date, status, created_at) VALUES (?, ?, 'stopped', ?)",
    )
    .bind(user_id)
    .bind(local_date.format("%Y-%m-%d").to_string())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;
    let session_id = done.last_insert_rowid();
    let opening_id = insert_event(pool, session_id, WorkEventKind::Started, start).await?;
    insert_event(pool, session_id, WorkEventKind::Stopped, end).await?;
    let events = event_records_for(pool, session_id).await?;
    labeled_intervals(session_id, &events, now)
        .into_iter()
        .find(|item| item.id == opening_id)
        .ok_or_else(|| AppError::Internal("Intervall nach dem Anlegen nicht gefunden".into()))
}

pub async fn update_interval(
    pool: &SqlitePool,
    user_id: i64,
    id: i64,
    start_at: DateTime<Utc>,
    end_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> AppResult<LabeledInterval> {
    let (session, interval, mut records) = find_owned_interval(pool, user_id, id, now).await?;
    let local_date = parse_date(&session.local_date)
        .map_err(|_| AppError::Internal("local_date ungültig".into()))?;

    if interval.open {
        if end_at.is_some() {
            return Err(AppError::Unprocessable(
                "Laufende Arbeitszeit hat kein Ende".into(),
            ));
        }
        let start = truncate_to_minute(start_at);
        if start >= now {
            return Err(AppError::Unprocessable(
                "Start muss vor dem aktuellen Zeitpunkt liegen".into(),
            ));
        }
        if civil_date(start, APP_TZ) != local_date {
            return Err(AppError::Unprocessable(
                "Tageswechsel ist nicht erlaubt".into(),
            ));
        }
        let candidate = Interval::running(start, now);
        ensure_no_work_overlap(pool, user_id, local_date, candidate, Some(id), now).await?;
        apply_event_times(&mut records, &[(interval.id, start)])?;
        persist_event_times(pool, &records).await?;
    } else {
        let end = end_at
            .ok_or_else(|| AppError::Unprocessable("Endzeitpunkt ist erforderlich".into()))?;
        let (start, end) = validate_closed(start_at, end, now, Some(local_date))?;
        let candidate = Interval::new(start, end)
            .ok_or_else(|| AppError::Unprocessable("Zeitraum ungültig".into()))?;
        ensure_no_work_overlap(pool, user_id, local_date, candidate, Some(id), now).await?;
        let mut updates = vec![(interval.id, start)];
        if let Some(close_id) = interval.close_event_id {
            updates.push((close_id, end));
        }
        apply_event_times(&mut records, &updates)?;
        persist_event_times(pool, &records).await?;
    }

    let events = event_records_for(pool, session.id).await?;
    labeled_intervals(session.id, &events, now)
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| AppError::Internal("Intervall nach dem Speichern nicht gefunden".into()))
}

pub async fn delete_interval(
    pool: &SqlitePool,
    user_id: i64,
    id: i64,
    now: DateTime<Utc>,
) -> AppResult<()> {
    let (session, interval, records) = find_owned_interval(pool, user_id, id, now).await?;
    if interval.open {
        return Err(AppError::Unprocessable(
            "Laufende Arbeitszeit kann nicht gelöscht werden".into(),
        ));
    }
    let remaining = remove_interval(&records, id).ok_or(AppError::NotFound)?;
    if remaining.is_empty() || labeled_intervals(session.id, &remaining, now).is_empty() {
        sqlx::query("DELETE FROM work_sessions WHERE id = ?")
            .bind(session.id)
            .execute(pool)
            .await?;
        return Ok(());
    }

    if let Some(close_id) = interval.close_event_id {
        delete_event(pool, close_id).await?;
    }
    delete_event(pool, interval.id).await?;
    if let Some(first) = remaining.first() {
        sqlx::query("UPDATE work_session_events SET kind = ? WHERE id = ?")
            .bind(kind_str(first.kind))
            .bind(first.id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

async fn ensure_timer_slot_free(
    pool: &SqlitePool,
    user_id: i64,
    local_date: NaiveDate,
    now: DateTime<Utc>,
) -> AppResult<()> {
    let existing = intervals_on(pool, user_id, local_date, now).await?;
    if existing.iter().any(|interval| interval.end > now) {
        return Err(AppError::Conflict(
            "Arbeitszeit überschneidet sich mit einem bestehenden Zeitraum".into(),
        ));
    }
    Ok(())
}

fn validate_closed(
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    now: DateTime<Utc>,
    expected_date: Option<NaiveDate>,
) -> AppResult<(DateTime<Utc>, DateTime<Utc>)> {
    let start = truncate_to_minute(start_at);
    let end = truncate_to_minute(end_at);
    if end <= start {
        return Err(AppError::Unprocessable(
            "Arbeitszeit braucht eine Dauer größer als 0".into(),
        ));
    }
    if start > now || end > now {
        return Err(AppError::Unprocessable(
            "Zeitpunkt liegt in der Zukunft".into(),
        ));
    }
    if !same_civil_day(start, end, APP_TZ) {
        return Err(AppError::Unprocessable(
            "Start und Ende müssen am selben Kalendertag liegen".into(),
        ));
    }
    let date = civil_date(start, APP_TZ);
    if expected_date.is_some_and(|expected| expected != date) {
        return Err(AppError::Unprocessable(
            "Tageswechsel ist nicht erlaubt".into(),
        ));
    }
    Ok((start, end))
}

async fn ensure_no_work_overlap(
    pool: &SqlitePool,
    user_id: i64,
    local_date: NaiveDate,
    candidate: Interval,
    except_id: Option<i64>,
    now: DateTime<Utc>,
) -> AppResult<()> {
    let existing = intervals_on(pool, user_id, local_date, now).await?;
    let others: Vec<Interval> = existing
        .iter()
        .filter(|interval| except_id != Some(interval.id))
        .filter_map(|interval| Interval::new(interval.start, interval.end))
        .collect();
    if any_overlap(candidate, &others) {
        return Err(AppError::Conflict(
            "Arbeitszeit überschneidet sich mit einem bestehenden Zeitraum".into(),
        ));
    }
    Ok(())
}

async fn find_owned_interval(
    pool: &SqlitePool,
    user_id: i64,
    id: i64,
    now: DateTime<Utc>,
) -> AppResult<(WorkSessionRow, LabeledInterval, Vec<WorkEventRecord>)> {
    let event = sqlx::query_as::<_, WorkEventRow>(
        "SELECT e.id, e.work_session_id, e.kind, e.at
         FROM work_session_events e
         INNER JOIN work_sessions s ON s.id = e.work_session_id
         WHERE e.id = ? AND s.user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let session = sqlx::query_as::<_, WorkSessionRow>(
        "SELECT id, user_id, local_date, status, created_at FROM work_sessions WHERE id = ?",
    )
    .bind(event.work_session_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let records = event_records_for(pool, session.id).await?;
    let interval = labeled_intervals(session.id, &records, now)
        .into_iter()
        .find(|item| item.id == id)
        .ok_or(AppError::NotFound)?;
    Ok((session, interval, records))
}

fn apply_event_times(
    records: &mut [WorkEventRecord],
    updates: &[(i64, DateTime<Utc>)],
) -> AppResult<()> {
    for (id, at) in updates {
        let event = records
            .iter_mut()
            .find(|event| event.id == *id)
            .ok_or_else(|| AppError::Internal("Event nicht gefunden".into()))?;
        event.at = *at;
    }
    if !timestamps_non_decreasing(records) {
        return Err(AppError::Unprocessable(
            "Die Zeiten der Arbeitszeit sind ungültig".into(),
        ));
    }
    Ok(())
}

async fn persist_event_times(pool: &SqlitePool, records: &[WorkEventRecord]) -> AppResult<()> {
    for event in records {
        sqlx::query("UPDATE work_session_events SET at = ? WHERE id = ?")
            .bind(event.at.to_rfc3339())
            .bind(event.id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

async fn delete_event(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM work_session_events WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
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

async fn event_records_for(pool: &SqlitePool, session_id: i64) -> AppResult<Vec<WorkEventRecord>> {
    let rows = sqlx::query_as::<_, WorkEventRow>(
        "SELECT id, work_session_id, kind, at FROM work_session_events WHERE work_session_id = ? ORDER BY id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let kind = parse_kind(&row.kind)?;
        let at =
            parse_rfc3339(&row.at).map_err(|_| AppError::Internal("Event-Zeit ungültig".into()))?;
        events.push(WorkEventRecord {
            id: row.id,
            kind,
            at,
        });
    }
    Ok(events)
}

fn parse_kind(value: &str) -> AppResult<WorkEventKind> {
    match value {
        "started" => Ok(WorkEventKind::Started),
        "paused" => Ok(WorkEventKind::Paused),
        "resumed" => Ok(WorkEventKind::Resumed),
        "stopped" => Ok(WorkEventKind::Stopped),
        _ => Err(AppError::Internal("Unbekanntes Work-Event".into())),
    }
}

fn kind_str(kind: WorkEventKind) -> &'static str {
    match kind {
        WorkEventKind::Started => "started",
        WorkEventKind::Paused => "paused",
        WorkEventKind::Resumed => "resumed",
        WorkEventKind::Stopped => "stopped",
    }
}

async fn insert_event(
    pool: &SqlitePool,
    session_id: i64,
    kind: WorkEventKind,
    at: DateTime<Utc>,
) -> AppResult<i64> {
    let done =
        sqlx::query("INSERT INTO work_session_events (work_session_id, kind, at) VALUES (?, ?, ?)")
            .bind(session_id)
            .bind(kind_str(kind))
            .bind(at.to_rfc3339())
            .execute(pool)
            .await?;
    Ok(done.last_insert_rowid())
}

async fn set_status(pool: &SqlitePool, id: i64, status: WorkSessionStatus) -> AppResult<()> {
    sqlx::query("UPDATE work_sessions SET status = ? WHERE id = ?")
        .bind(status.as_str())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

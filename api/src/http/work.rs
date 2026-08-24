use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::LabeledInterval;
use crate::error::AppResult;
use crate::http::extractors::CurrentUser;
use crate::services::work::{self, WorkSnapshot};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListParams {
    from: NaiveDate,
    to: NaiveDate,
}

#[derive(Serialize)]
pub struct WorkView {
    session_id: Option<i64>,
    status: Option<String>,
    local_date: String,
    elapsed_seconds: i64,
}

impl From<WorkSnapshot> for WorkView {
    fn from(snap: WorkSnapshot) -> Self {
        Self {
            session_id: snap.session.as_ref().map(|row| row.id),
            status: snap.session.as_ref().map(|row| row.status.clone()),
            local_date: snap.local_date.format("%Y-%m-%d").to_string(),
            elapsed_seconds: snap.elapsed_seconds,
        }
    }
}

#[derive(Serialize)]
pub struct WorkIntervalView {
    id: i64,
    start_at: String,
    end_at: String,
    open: bool,
}

impl From<&LabeledInterval> for WorkIntervalView {
    fn from(interval: &LabeledInterval) -> Self {
        Self {
            id: interval.id,
            start_at: interval.start.to_rfc3339(),
            end_at: interval.end.to_rfc3339(),
            open: interval.open,
        }
    }
}

#[derive(Serialize)]
pub struct WorkDayView {
    local_date: String,
    elapsed_seconds: i64,
    intervals: Vec<WorkIntervalView>,
}

pub async fn list(
    user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> AppResult<Json<Vec<WorkDayView>>> {
    let days =
        work::list_for_range(&state.pool, user.id, params.from, params.to, Utc::now()).await?;
    Ok(Json(
        days.into_iter()
            .map(|day| WorkDayView {
                local_date: day.local_date.format("%Y-%m-%d").to_string(),
                elapsed_seconds: day.elapsed_seconds,
                intervals: day.intervals.iter().map(WorkIntervalView::from).collect(),
            })
            .collect(),
    ))
}

pub async fn current(
    user: CurrentUser,
    State(state): State<AppState>,
) -> AppResult<Json<WorkView>> {
    let snap = work::current(&state.pool, user.id, Utc::now()).await?;
    Ok(Json(WorkView::from(snap)))
}

pub async fn start(user: CurrentUser, State(state): State<AppState>) -> AppResult<Json<WorkView>> {
    let snap = work::start(&state.pool, user.id, Utc::now()).await?;
    Ok(Json(WorkView::from(snap)))
}

pub async fn pause(user: CurrentUser, State(state): State<AppState>) -> AppResult<Json<WorkView>> {
    let snap = work::pause(&state.pool, user.id, Utc::now()).await?;
    Ok(Json(WorkView::from(snap)))
}

pub async fn resume(user: CurrentUser, State(state): State<AppState>) -> AppResult<Json<WorkView>> {
    let snap = work::resume(&state.pool, user.id, Utc::now()).await?;
    Ok(Json(WorkView::from(snap)))
}

pub async fn stop(user: CurrentUser, State(state): State<AppState>) -> AppResult<Json<WorkView>> {
    let snap = work::stop(&state.pool, user.id, Utc::now()).await?;
    Ok(Json(WorkView::from(snap)))
}

#[derive(Deserialize)]
pub struct CreateIntervalBody {
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct PatchIntervalBody {
    start_at: DateTime<Utc>,
    end_at: Option<DateTime<Utc>>,
}

pub async fn create_interval(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<CreateIntervalBody>,
) -> AppResult<Json<WorkIntervalView>> {
    let interval =
        work::create_interval(&state.pool, user.id, body.start_at, body.end_at, Utc::now()).await?;
    Ok(Json(WorkIntervalView::from(&interval)))
}

pub async fn update_interval(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PatchIntervalBody>,
) -> AppResult<Json<WorkIntervalView>> {
    let interval = work::update_interval(
        &state.pool,
        user.id,
        id,
        body.start_at,
        body.end_at,
        Utc::now(),
    )
    .await?;
    Ok(Json(WorkIntervalView::from(&interval)))
}

pub async fn delete_interval(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    work::delete_interval(&state.pool, user.id, id, Utc::now()).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

use axum::Json;
use axum::extract::{Query, State};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

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
pub struct WorkDayView {
    local_date: String,
    elapsed_seconds: i64,
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

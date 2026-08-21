use axum::Json;
use axum::extract::State;
use chrono::Utc;
use serde::Serialize;

use crate::error::AppResult;
use crate::http::extractors::CurrentUser;
use crate::services::work::{self, WorkSnapshot};
use crate::state::AppState;

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

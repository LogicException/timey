use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::http::extractors::CurrentUser;
use crate::models::EntryRow;
use crate::services::entries::{self, EntryFilters, NewEntry};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListParams {
    from: NaiveDate,
    to: NaiveDate,
    #[serde(default)]
    task_ids: String,
    #[serde(default)]
    project_ids: String,
    #[serde(default)]
    aufgabe_ids: String,
}

#[derive(Deserialize)]
pub struct EntryBody {
    task_id: Option<i64>,
    project_id: Option<i64>,
    aufgabe_id: Option<i64>,
    start_at: DateTime<Utc>,
    end_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct StopBody {
    task_id: i64,
    project_id: Option<i64>,
    aufgabe_id: Option<i64>,
}

#[derive(Serialize)]
pub struct EntryView {
    id: i64,
    task_id: Option<i64>,
    project_id: Option<i64>,
    aufgabe_id: Option<i64>,
    task_name: Option<String>,
    project_name: Option<String>,
    aufgabe_name: Option<String>,
    start_at: String,
    end_at: Option<String>,
    status: String,
}

impl From<EntryRow> for EntryView {
    fn from(row: EntryRow) -> Self {
        Self {
            id: row.id,
            task_id: row.task_id,
            project_id: row.project_id,
            aufgabe_id: row.aufgabe_id,
            task_name: row.task_name,
            project_name: row.project_name,
            aufgabe_name: row.aufgabe_name,
            start_at: row.start_at,
            end_at: row.end_at,
            status: row.status,
        }
    }
}

pub async fn list_entries(
    user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> AppResult<Json<Vec<EntryView>>> {
    let filters = parse_filters(&params);
    let rows =
        entries::list_entries(&state.pool, user.id, params.from, params.to, &filters).await?;
    Ok(Json(rows.into_iter().map(EntryView::from).collect()))
}

pub async fn export_csv(
    user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> AppResult<impl IntoResponse> {
    let filters = parse_filters(&params);
    let rows =
        entries::list_entries(&state.pool, user.id, params.from, params.to, &filters).await?;
    let csv = entries::entries_to_csv(&rows)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"timey.csv\""),
    );
    Ok((StatusCode::OK, headers, csv))
}

pub async fn create_entry(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<EntryBody>,
) -> AppResult<Json<EntryView>> {
    let row = entries::create_entry(
        &state.pool,
        user.id,
        NewEntry {
            task_id: body.task_id,
            project_id: body.project_id,
            aufgabe_id: body.aufgabe_id,
            start_at: body.start_at,
            end_at: body.end_at,
        },
        Utc::now(),
    )
    .await?;
    Ok(Json(EntryView::from(row)))
}

pub async fn update_entry(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<EntryBody>,
) -> AppResult<Json<EntryView>> {
    let row = entries::update_entry(
        &state.pool,
        user.id,
        id,
        NewEntry {
            task_id: body.task_id,
            project_id: body.project_id,
            aufgabe_id: body.aufgabe_id,
            start_at: body.start_at,
            end_at: body.end_at,
        },
        Utc::now(),
    )
    .await?;
    Ok(Json(EntryView::from(row)))
}

pub async fn delete_entry(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    entries::delete_entry(&state.pool, user.id, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_timer(
    user: CurrentUser,
    State(state): State<AppState>,
) -> AppResult<Json<Option<EntryView>>> {
    let row = entries::running_entry(&state.pool, user.id).await?;
    Ok(Json(row.map(EntryView::from)))
}

pub async fn start_timer(
    user: CurrentUser,
    State(state): State<AppState>,
) -> AppResult<Json<EntryView>> {
    let row = entries::start_timer(&state.pool, user.id, Utc::now()).await?;
    Ok(Json(EntryView::from(row)))
}

pub async fn stop_timer(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<StopBody>,
) -> AppResult<Json<EntryView>> {
    if body.task_id == 0 {
        return Err(AppError::Unprocessable("Task ist erforderlich".into()));
    }
    let row = entries::stop_timer(
        &state.pool,
        user.id,
        body.task_id,
        body.project_id,
        body.aufgabe_id,
        Utc::now(),
    )
    .await?;
    Ok(Json(EntryView::from(row)))
}

fn parse_filters(params: &ListParams) -> EntryFilters {
    EntryFilters {
        task_ids: parse_ids(&params.task_ids),
        project_ids: parse_ids(&params.project_ids),
        aufgabe_ids: parse_ids(&params.aufgabe_ids),
    }
}

fn parse_ids(raw: &str) -> Vec<i64> {
    raw.split(',')
        .filter_map(|part| {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse().ok()
            }
        })
        .collect()
}

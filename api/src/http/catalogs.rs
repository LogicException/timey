use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::http::extractors::{CurrentUser, require_admin};
use crate::models::{NamedRow, ProjectRow};
use crate::services::catalogs;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    include_archived: bool,
    #[serde(default)]
    include_system: bool,
}

#[derive(Deserialize)]
pub struct NameBody {
    name: String,
}

#[derive(Deserialize)]
pub struct ArchiveBody {
    archived: bool,
}

#[derive(Deserialize)]
pub struct PatchTaskBody {
    name: Option<String>,
    archived: Option<bool>,
    color: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectView {
    id: i64,
    name: String,
    archived: bool,
}

impl From<ProjectRow> for ProjectView {
    fn from(row: ProjectRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            archived: row.archived,
        }
    }
}

#[derive(Serialize)]
pub struct NamedView {
    id: i64,
    name: String,
    archived: bool,
    system: bool,
    color: String,
}

impl From<NamedRow> for NamedView {
    fn from(row: NamedRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            archived: row.archived,
            system: row.is_system,
            color: row.color,
        }
    }
}

pub async fn list_projects(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<ProjectView>>> {
    let rows = catalogs::list_projects(&state.pool, query.include_archived).await?;
    Ok(Json(rows.into_iter().map(ProjectView::from).collect()))
}

pub async fn create_project(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<NameBody>,
) -> AppResult<Json<ProjectView>> {
    require_admin(&user)?;
    let row =
        catalogs::create_project(&state.pool, &body.name, user.id, chrono::Utc::now()).await?;
    Ok(Json(ProjectView::from(row)))
}

pub async fn patch_project(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ArchiveBody>,
) -> AppResult<Json<ProjectView>> {
    require_admin(&user)?;
    let row = catalogs::set_project_archived(&state.pool, id, body.archived).await?;
    Ok(Json(ProjectView::from(row)))
}

pub async fn list_tasks(
    user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<NamedView>>> {
    let rows = catalogs::list_tasks(
        &state.pool,
        user.id,
        query.include_archived,
        query.include_system,
    )
    .await?;
    Ok(Json(rows.into_iter().map(NamedView::from).collect()))
}

pub async fn create_task(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<NameBody>,
) -> AppResult<Json<NamedView>> {
    let row = catalogs::create_task(&state.pool, user.id, &body.name, chrono::Utc::now()).await?;
    Ok(Json(NamedView::from(row)))
}

pub async fn patch_task(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PatchTaskBody>,
) -> AppResult<Json<NamedView>> {
    if body.name.is_none() && body.archived.is_none() && body.color.is_none() {
        return Err(AppError::Unprocessable(
            "name, archived oder color ist erforderlich".into(),
        ));
    }
    if let Some(name) = body.name.as_deref() {
        catalogs::rename_task(&state.pool, user.id, id, name).await?;
    }
    if let Some(archived) = body.archived {
        catalogs::set_task_archived(&state.pool, user.id, id, archived).await?;
    }
    if let Some(color) = body.color.as_deref() {
        catalogs::set_task_color(&state.pool, user.id, id, color).await?;
    }
    let row = catalogs::get_task(&state.pool, user.id, id).await?;
    Ok(Json(NamedView::from(row)))
}

pub async fn delete_task(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    catalogs::delete_task(&state.pool, user.id, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

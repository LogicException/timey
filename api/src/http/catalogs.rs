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
}

impl From<NamedRow> for NamedView {
    fn from(row: NamedRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            archived: row.archived,
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
    let rows = catalogs::list_tasks(&state.pool, user.id, query.include_archived).await?;
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
    match (body.name.as_deref(), body.archived) {
        (None, None) => Err(AppError::Unprocessable(
            "name oder archived ist erforderlich".into(),
        )),
        (Some(name), None) => {
            let row = catalogs::rename_task(&state.pool, user.id, id, name).await?;
            Ok(Json(NamedView::from(row)))
        }
        (None, Some(archived)) => {
            let row = catalogs::set_task_archived(&state.pool, user.id, id, archived).await?;
            Ok(Json(NamedView::from(row)))
        }
        (Some(name), Some(archived)) => {
            catalogs::rename_task(&state.pool, user.id, id, name).await?;
            let row = catalogs::set_task_archived(&state.pool, user.id, id, archived).await?;
            Ok(Json(NamedView::from(row)))
        }
    }
}

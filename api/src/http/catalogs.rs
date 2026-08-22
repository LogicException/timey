use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::http::extractors::{CurrentUser, require_admin};
use crate::models::{NamedRow, ProjectRow};
use crate::services::catalogs::{self, UserItemTable};
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
    list_items(user, state, query, UserItemTable::Task).await
}

pub async fn create_task(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<NameBody>,
) -> AppResult<Json<NamedView>> {
    create_item(user, state, body, UserItemTable::Task).await
}

pub async fn patch_task(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ArchiveBody>,
) -> AppResult<Json<NamedView>> {
    patch_item(user, state, id, body, UserItemTable::Task).await
}

pub async fn list_aufgaben(
    user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<NamedView>>> {
    list_items(user, state, query, UserItemTable::Aufgabe).await
}

pub async fn create_aufgabe(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<NameBody>,
) -> AppResult<Json<NamedView>> {
    create_item(user, state, body, UserItemTable::Aufgabe).await
}

pub async fn patch_aufgabe(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ArchiveBody>,
) -> AppResult<Json<NamedView>> {
    patch_item(user, state, id, body, UserItemTable::Aufgabe).await
}

async fn list_items(
    user: CurrentUser,
    state: AppState,
    query: ListQuery,
    table: UserItemTable,
) -> AppResult<Json<Vec<NamedView>>> {
    let rows =
        catalogs::list_user_items(&state.pool, table, user.id, query.include_archived).await?;
    Ok(Json(rows.into_iter().map(NamedView::from).collect()))
}

async fn create_item(
    user: CurrentUser,
    state: AppState,
    body: NameBody,
    table: UserItemTable,
) -> AppResult<Json<NamedView>> {
    let row =
        catalogs::create_user_item(&state.pool, table, user.id, &body.name, chrono::Utc::now())
            .await?;
    Ok(Json(NamedView::from(row)))
}

async fn patch_item(
    user: CurrentUser,
    state: AppState,
    id: i64,
    body: ArchiveBody,
    table: UserItemTable,
) -> AppResult<Json<NamedView>> {
    let row =
        catalogs::set_user_item_archived(&state.pool, table, user.id, id, body.archived).await?;
    Ok(Json(NamedView::from(row)))
}

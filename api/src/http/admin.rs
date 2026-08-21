use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::http::extractors::{require_admin, CurrentUser};
use crate::models::{Role, UserRow};
use crate::services::users;
use crate::state::AppState;

#[derive(Serialize)]
pub struct AdminUserView {
    id: i64,
    username: String,
    role: String,
    disabled: bool,
    auth_provider: String,
}

impl From<UserRow> for AdminUserView {
    fn from(user: UserRow) -> Self {
        Self {
            id: user.id,
            username: user.username,
            role: user.role,
            disabled: user.disabled,
            auth_provider: user.auth_provider,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
    password: String,
    role: Role,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    password: Option<String>,
    role: Option<Role>,
    disabled: Option<bool>,
}

pub async fn list_users(
    user: CurrentUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<AdminUserView>>> {
    require_admin(&user)?;
    let users = users::list_users(&state.pool).await?;
    Ok(Json(users.into_iter().map(AdminUserView::from).collect()))
}

pub async fn create_user(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> AppResult<Json<AdminUserView>> {
    require_admin(&user)?;
    let created = users::create_user(
        &state.pool,
        &body.username,
        &body.password,
        body.role,
        chrono::Utc::now(),
    )
    .await?;
    Ok(Json(AdminUserView::from(created)))
}

pub async fn update_user(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateUserRequest>,
) -> AppResult<Json<AdminUserView>> {
    require_admin(&user)?;
    let updated = users::update_user(
        &state.pool,
        id,
        body.password.as_deref(),
        body.role,
        body.disabled,
    )
    .await?;
    Ok(Json(AdminUserView::from(updated)))
}
